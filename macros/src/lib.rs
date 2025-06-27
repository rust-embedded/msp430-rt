extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate rand;
extern crate rand_xoshiro;
extern crate syn;

use proc_macro::TokenStream;
use std::{
    collections::HashSet,
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use proc_macro2::Span;
use quote::{quote, quote_spanned};
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use syn::{
    parenthesized,
    parse::{self, Parse},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    FnArg, Ident, Item, ItemFn, ItemStatic, Pat, PatIdent, PathArguments, PathSegment, ReturnType,
    Stmt, Token, Type, TypePath, Visibility,
};

/// Attribute to declare the entry point of the program
///
/// The specified function will be called by the reset handler *after* RAM has been initialized.
///
/// The type of the specified function must be `[unsafe] fn([<name>: CriticalSection]) -> !` (never
/// ending function), where the `CriticalSection` argument is optional.
///
/// # Properties
///
/// The entry point will be called by the reset handler. The program can't reference to the entry
/// point, much less invoke it.
///
/// `static mut` variables declared within the entry point are safe to access. The compiler can't
/// prove this is safe so the attribute will help by making a transformation to the source code: for
/// this reason a variable like `static mut FOO: u32` will become `let FOO: &'static mut u32;`. Note
/// that `&'static mut` references have move semantics.
///
/// ## Examples
///
/// - Simple entry point
///
/// ``` no_run
/// # #![no_main]
/// # use msp430_rt_macros::entry;
/// #[entry]
/// fn main() -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
///
/// - `static mut` variables local to the entry point are safe to modify.
///
/// ``` no_run
/// # #![no_main]
/// # use msp430_rt_macros::entry;
/// #[entry]
/// fn main(_cs: CriticalSection) -> ! {
///     static mut FOO: u32 = 0;
///
///     let foo: &'static mut u32 = FOO;
///     assert_eq!(*foo, 0);
///     *foo = 1;
///     assert_eq!(*foo, 1);
///
///     loop {
///         /* .. */
///     }
/// }
/// ```
///
/// # Pre-entry Interrupt Enable
///
/// If the argument `interrupt_enable` is passed into the macro, interrupts will be enabled
/// globally before the entry function runs. If this is enabled then the entry function will no
/// longer accept `CriticalSection` as a parameter, since that it will be unsound.
///
/// The macro can also take arguments of the form `interrupt_enable(pre_interrupt = <init>)`, where
/// `init` is the name of a function with the signature `fn(cs: CriticalSection) -> <Type>`.  The
/// entry function can then optionally take a parameter of `Type`. This makes `init` run before
/// interrupts are enabled and possibly pass its return value into the entry function, allowing
/// pre-interrupt initialization to be done.
///
/// Note that a function marked with the entry attribute is allowed to take no input parameters
/// even if `init` returns a value, due to implementation details. To reduce code size, it is
/// strongly recommended to put `#[inline(always)]` on `init` if it's used nowhere else.
///
/// ## Examples
///
/// - Enable interrupts before entry
///
/// ``` no_run
/// # #![no_main]
/// # use msp430_rt_macros::entry;
/// #[entry(interrupt_enable)]
/// fn main() -> ! {
///     /* interrupts now enabled */
///     loop {}
/// }
/// ```
///
/// - Pre-interrupt initialization
///
/// ``` no_run
/// # #![no_main]
/// # use msp430_rt_macros::entry;
/// use msp430::interrupt::CriticalSection;
///
/// # struct Hal;
/// #[inline(always)]
/// fn init(cs: CriticalSection) -> Hal {
///     /* initialize hardware */
///     # Hal
/// }
///
/// #[entry(interrupt_enable(pre_interrupt = init))]
/// fn main(hal: Hal) -> ! {
///     loop {
///         /* do something with hal */
///     }
/// }
/// ```
///
/// - Pre-interrupt initialization with no return
///
/// ``` no_run
/// # #![no_main]
/// # use msp430_rt_macros::entry;
/// use msp430::interrupt::CriticalSection;
///
/// #[inline(always)]
/// fn arg(cs: CriticalSection) {
///     /* initialize */
/// }
///
/// #[entry(interrupt_enable(pre_interrupt = arg))]
/// fn main() -> ! {
///     loop {}
/// }
/// ```
///
/// ## Note
///
/// The `CriticalSection`s passed into the entry and the pre-interrupt functions have their
/// lifetimes restrained to their respective functions. Attempting to pass the `CriticalSection`
/// outside its scope fails with a `borrowed value does not live long enough` error.
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let interrupt_enable = if args.is_empty() {
        None
    } else {
        Some(parse_macro_input!(args as EntryInterruptEnable))
    };

    let f = parse_macro_input!(input as ItemFn);

    // check the function signature
    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
        };

    let pair = match &interrupt_enable {
        Some(interrupt_enable) => interrupt_enable.extract_init_arg(&f.sig.inputs),
        None => extract_critical_section_arg(&f.sig.inputs),
    };

    if let (true, Ok(ParamArgPair { fn_param, fn_arg })) = (valid_signature, pair) {
        // XXX should we blacklist other attributes?
        let attrs = f.attrs;
        let unsafety = f.sig.unsafety;
        let hash = random_ident();
        let (statics, stmts) = match extract_static_muts(f.block.stmts) {
            Err(e) => return e.to_compile_error().into(),
            Ok(x) => x,
        };

        let vars = statics
            .into_iter()
            .map(|var| {
                let attrs = var.attrs;
                let ident = var.ident;
                let ty = var.ty;
                let expr = var.expr;

                quote!(
                    #[allow(non_snake_case)]
                    let #ident: &'static mut #ty = unsafe {
                        #(#attrs)*
                        static mut #ident: #ty = #expr;

                        &mut #ident
                    };
                )
            })
            .collect::<Vec<_>>();

        // Only generate the argument if fn_param exists, to handle the case where the argument
        // expression does exist but the entry function doesn't accept any args
        let arg_ident = fn_param
            .as_ref()
            .map(|_| Ident::new("arg", Span::mixed_site()));
        let arg_def = fn_arg
            .as_ref()
            .map(|arg| quote_spanned!(Span::mixed_site()=> let arg = #arg; ));

        quote!(
            #[export_name = "main"]
            #(#attrs)*
            pub #unsafety fn #hash() -> ! {
                #[inline(always)]
                #unsafety fn #hash<'a>(#fn_param) -> ! {
                    #(#vars)*
                    #(#stmts)*
                }
                #arg_def
                { #hash(#arg_ident) }
            }
        )
        .into()
    } else {
        let err = match interrupt_enable {
            None => parse::Error::new(
                f.sig.span(),
                "`#[entry]` function must have signature `[unsafe] fn([<ident> : CriticalSection]) -> !`",
            ),
            Some(EntryInterruptEnable { pre_interrupt: None }) => parse::Error::new(
                f.sig.span(),
                "`#[entry(interrupt_enable)]` function must have signature `[unsafe] fn() -> !`",
            ),
            Some(EntryInterruptEnable { pre_interrupt: Some(ident) }) => parse::Error::new(
                f.sig.span(),
                format!("`#[entry(interrupt_enable(pre_interrupt = {fname}))]` function must have signature `[unsafe] fn([<ident> : <Type>]) -> !`, where <Type> is the return value of {fname}", fname = ident)
            ),
        };
        err.to_compile_error().into()
    }
}

#[derive(Default)]
struct ParamArgPair {
    fn_param: Option<proc_macro2::TokenStream>,
    fn_arg: Option<proc_macro2::TokenStream>,
}

struct EntryInterruptEnable {
    pre_interrupt: Option<Ident>,
}

impl Parse for EntryInterruptEnable {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let interrupt_enable = input.parse::<Ident>()?;
        if interrupt_enable != "interrupt_enable" {
            return Err(parse::Error::new(
                interrupt_enable.span(),
                "expected `interrupt_enable` or no arguments at all",
            ));
        }
        let pre_interrupt = if input.peek(syn::token::Paren) {
            let inner;
            parenthesized!(inner in input);
            let pre_interrupt = inner.parse::<Ident>()?;
            if pre_interrupt != "pre_interrupt" {
                return Err(parse::Error::new(
                    pre_interrupt.span(),
                    "expected `pre_interrupt`",
                ));
            }
            inner.parse::<syn::token::Eq>()?;
            Some(inner.parse::<Ident>()?)
        } else {
            None
        };

        Ok(EntryInterruptEnable { pre_interrupt })
    }
}

impl EntryInterruptEnable {
    fn extract_init_arg(&self, list: &Punctuated<FnArg, Token![,]>) -> Result<ParamArgPair, ()> {
        if let Some(fn_name) = &self.pre_interrupt {
            let hash = random_ident();
            let fn_arg = Some(quote_spanned!(Span::mixed_site()=> {
                let cs = unsafe { msp430::interrupt::CriticalSection::new() };

                // This struct forces the lifetime of the CriticalSection to match the lifetime of
                // the reference. Since the reference lifetime is restricted to this scope, the
                // compiler has to constrain the lifetime of the CriticalSection as well,
                // preventing the CriticalSection from being leaked as a return value.
                #[allow(non_camel_case_types)]
                struct #hash<'a>(&'a msp430::interrupt::CriticalSection<'a>);
                let arg = #fn_name(*#hash(&cs).0);

                unsafe { msp430::interrupt::enable() };
                arg
            }));

            if let Some(first) = list.first() {
                if let FnArg::Typed(pat_type) = first {
                    // Case where pre-init exists and entry takes a param
                    return Ok(ParamArgPair {
                        fn_param: Some(quote! { #pat_type }),
                        fn_arg,
                    });
                }
            } else {
                // Case where pre-init exists but entry takes no params
                return Ok(ParamArgPair {
                    fn_param: None,
                    fn_arg,
                });
            }
        } else if list.is_empty() {
            // Case where pre-init doesn't exist and entry takes no params
            return Ok(ParamArgPair {
                fn_param: None,
                fn_arg: Some(quote!({
                    unsafe { msp430::interrupt::enable() };
                })),
            });
        }
        Err(())
    }
}

/// Attribute to declare an interrupt handler
///
/// When the `device` feature is disabled this attribute can only be used to override the
/// DefaultHandler.
///
/// When the `device` feature is enabled this attribute can be used to override other interrupt
/// handlers but only when imported from a PAC (Peripheral Access Crate) crate which re-exports it.
/// Importing this attribute from the `msp430-rt` crate and using it on a function will result in a
/// compiler error.
///
/// # Syntax
///
/// ``` ignore
/// extern crate device;
///
/// // the attribute comes from the device crate not from msp430-rt
/// use device::interrupt;
///
/// #[interrupt]
/// // Pass in optional CriticalSection
/// fn USART1(cs: CriticalSection) {
///     // ..
/// }
/// ```
///
/// where the name of the function must be `DefaultHandler` or one of the device interrupts.
///
/// # Usage
///
/// `#[interrupt] fn Name(..` overrides the default handler for the interrupt with the given `Name`.
/// These handlers must have signature `[unsafe] fn([<name>: CriticalSection]) [-> !]`. It's
/// possible to add state to these handlers by declaring `static mut` variables at the beginning of
/// the body of the function. These variables will be safe to access from the function body.
///
/// If the interrupt handler has not been overridden it will be dispatched by the default interrupt
/// handler (`DefaultHandler`).
///
/// `#[interrupt] fn DefaultHandler(..` can be used to override the default interrupt handler. When
/// not overridden `DefaultHandler` defaults to an infinite loop.
///
/// `#[interrupt(wake_cpu)]` additionally returns the CPU to active mode after the interrupt
/// returns. This cannot be done by naively writing to the status register, as the status register
/// contents are pushed to the stack before an interrupt begins and this value is loaded back into
/// the status register after an interrupt completes, effectively making any changes to the status
/// register within an interrupt temporary.
/// Using the `wake_cpu` variant incurs a delay of two instructions (6 cycles) before the interrupt
/// handler begins.
/// The following status register bits are cleared: SCG1, SCG0, OSC_OFF and CPU_OFF.
///
/// # Properties
///
/// Interrupts handlers can only be called by the hardware. Other parts of the program can't refer
/// to the interrupt handlers, much less invoke them as if they were functions.
///
/// `static mut` variables declared within an interrupt handler are safe to access and can be used
/// to preserve state across invocations of the handler. The compiler can't prove this is safe so
/// the attribute will help by making a transformation to the source code: for this reason a
/// variable like `static mut FOO: u32` will become `let FOO: &mut u32;`.
///
/// ## Examples
///
/// - Using state within an interrupt handler
///
/// ``` ignore
/// extern crate device;
///
/// use device::interrupt;
///
/// #[interrupt]
/// fn TIM2() {
///     static mut COUNT: i32 = 0;
///
///     // `COUNT` is safe to access and has type `&mut i32`
///     *COUNT += 1;
///
///     println!("{}", COUNT);
/// }
/// ```
///
/// ## Note
///
/// The `CriticalSection` passed into the interrupt function has its lifetime restrained to the
/// function scope. Attempting to pass the `CriticalSection` outside its scope fails with a
/// `borrowed value does not live long enough` error.
#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse(input).expect("`#[interrupt]` must be applied to a function");

    let maybe_arg = parse_macro_input::parse::<Option<Ident>>(args.clone());

    let wake_cpu = match maybe_arg {
        Ok(None) => false,
        Ok(Some(ident)) if ident == "wake_cpu" => true,
        Ok(Some(_)) => {
            return parse::Error::new(
                Span::call_site(),
                "this attribute accepts only 'wake_cpu' as an argument",
            )
            .to_compile_error()
            .into()
        }
        Err(e) => return e.into_compile_error().into(),
    };

    let fspan = f.sig.span();
    let ident = f.sig.ident;

    let check = if ident == "DefaultHandler" {
        None
    } else if cfg!(feature = "device") {
        Some(quote!(interrupt::#ident;))
    } else {
        return parse::Error::new(
            ident.span(),
            "only the DefaultHandler can be overridden when the `device` feature is disabled",
        )
        .to_compile_error()
        .into();
    };

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let block = f.block;
    let stmts = block.stmts;
    let unsafety = f.sig.unsafety;

    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ref ty) => match **ty {
                Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                Type::Never(..) => true,
                _ => false,
            },
        };

    let pair = extract_critical_section_arg(&f.sig.inputs);

    if let (true, Ok(ParamArgPair { fn_arg, fn_param })) = (valid_signature, pair) {
        let (statics, stmts) = match extract_static_muts(stmts) {
            Err(e) => return e.to_compile_error().into(),
            Ok(x) => x,
        };

        let vars = statics
            .into_iter()
            .map(|var| {
                let attrs = var.attrs;
                let ident = var.ident;
                let ty = var.ty;
                let expr = var.expr;

                quote!(
                    #[allow(non_snake_case)]
                    let #ident: &mut #ty = unsafe {
                        #(#attrs)*
                        static mut #ident: #ty = #expr;

                        &mut #ident
                    };
                )
            })
            .collect::<Vec<_>>();

        let output = f.sig.output;
        let hash = random_ident();
        let ident = ident.to_string();
        if wake_cpu {
            quote!(
                #[export_name = #ident]
                #(#attrs)*
                #[unsafe(naked)]
                unsafe extern "msp430-interrupt" fn #hash() {
                    #[inline(always)]
                    #unsafety extern "msp430-interrupt" fn #hash<'a>(#fn_param) #output {
                        #check
                        #(#vars)*
                        #(#stmts)*
                    }
                    {
                        // Clear SCG1, SCG0, OSC_OFF, CPU_OFF in saved copy of SR register on stack
                        const MASK: u8 = (1<<7) + (1<<6) + (1<<5) + (1<<4);
                        core::arch::naked_asm!(
                            "bic.b #{mask}, 0(r1)",
                            "jmp {inner}",
                            inner = sym #hash,
                            mask = const MASK
                        );
                    }
                }
            )
        } else {
            quote!(
                #[export_name = #ident]
                #(#attrs)*
                #unsafety extern "msp430-interrupt" fn #hash() {
                    #check

                    #[inline(always)]
                    #unsafety fn #hash<'a>(#fn_param) #output {
                        #(#vars)*
                        #(#stmts)*
                    }
                    { #hash(#fn_arg) }
                }
            )
        }.into()
    } else {
        parse::Error::new(
            fspan,
            "`#[interrupt]` handlers must have signature `[unsafe] fn([<name>: CriticalSection]) [-> !]`",
        )
        .to_compile_error()
        .into()
    }
}

/// Attribute to mark which function will be called at the beginning of the reset handler.
///
/// **IMPORTANT**: This attribute can appear at most *once* in the dependency graph.
///
/// The function must have the signature of `unsafe fn()`.
///
/// The function passed will be called before static variables are initialized. Any access of static
/// variables will result in undefined behavior.
///
/// ## Examples
///
/// ```
/// # use msp430_rt_macros::pre_init;
/// #[pre_init]
/// unsafe fn before_main() {
///     // do something here
/// }
///
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn pre_init(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    // check the function signature
    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.unsafety.is_some()
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ref ty) => match **ty {
                Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                _ => false,
            },
        };

    if !valid_signature {
        return parse::Error::new(
            f.sig.span(),
            "`#[pre_init]` function must have signature `unsafe fn()`",
        )
        .to_compile_error()
        .into();
    }

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "this attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let ident = f.sig.ident;
    let block = f.block;

    quote!(
        #[export_name = "__pre_init"]
        #(#attrs)*
        pub unsafe fn #ident() #block
    )
    .into()
}

// Parses an optional `<name>: CriticalSection` from a list of function arguments.
// Additional arguments are considered invalid
fn extract_critical_section_arg(list: &Punctuated<FnArg, Token![,]>) -> Result<ParamArgPair, ()> {
    let num_args = list.len();
    if num_args == 0 {
        return Ok(ParamArgPair::default());
    } else if num_args == 1 {
        if let FnArg::Typed(pat_type) = list.first().unwrap() {
            if let (
                Pat::Ident(PatIdent {
                    ident: name,
                    by_ref: None,
                    mutability: None,
                    subpat: None,
                    attrs,
                }),
                Type::Path(TypePath { qself: None, path }),
                _,
                [],
            ) = (
                &*pat_type.pat,
                &*pat_type.ty,
                pat_type.colon_token,
                &*pat_type.attrs,
            ) {
                if path.segments.len() == 1 && attrs.is_empty() {
                    let seg = path.segments.first().unwrap();
                    if matches!(
                        seg,
                        PathSegment {
                            ident,
                            arguments: PathArguments::None,
                        } if ident == "CriticalSection"
                    ) {
                        return Ok(ParamArgPair {
                            fn_param: Some(
                                quote! { #name: msp430::interrupt::CriticalSection<'a> },
                            ),
                            fn_arg: Some(
                                quote! { unsafe { msp430::interrupt::CriticalSection::new() } },
                            ),
                        });
                    }
                }
            }
        }
    }
    Err(())
}

// Creates a random identifier
fn random_ident() -> Ident {
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let count: u64 = CALL_COUNT.fetch_add(1, Ordering::SeqCst) as u64;
    let mut seed: [u8; 16] = [0; 16];

    for (i, v) in seed.iter_mut().take(8).enumerate() {
        *v = ((secs >> (i * 8)) & 0xFF) as u8
    }

    for (i, v) in seed.iter_mut().skip(8).enumerate() {
        *v = ((count >> (i * 8)) & 0xFF) as u8
    }

    let mut rng = rand_xoshiro::Xoshiro128PlusPlus::from_seed(seed);
    Ident::new(
        &(0..16)
            .map(|i| {
                if i == 0 || rng.gen() {
                    (b'a' + rng.gen::<u8>() % 25) as char
                } else {
                    (b'0' + rng.gen::<u8>() % 10) as char
                }
            })
            .collect::<String>(),
        Span::call_site(),
    )
}

/// Extracts `static mut` vars from the beginning of the given statements
fn extract_static_muts(stmts: Vec<Stmt>) -> Result<(Vec<ItemStatic>, Vec<Stmt>), parse::Error> {
    let mut istmts = stmts.into_iter();

    let mut seen = HashSet::new();
    let mut statics = vec![];
    let mut stmts = vec![];
    for stmt in istmts.by_ref() {
        match stmt {
            Stmt::Item(Item::Static(var)) => {
                if var.mutability.is_some() {
                    if seen.contains(&var.ident) {
                        return Err(parse::Error::new(
                            var.ident.span(),
                            format!("the name `{}` is defined multiple times", var.ident),
                        ));
                    }

                    seen.insert(var.ident.clone());
                    statics.push(var);
                } else {
                    stmts.push(Stmt::Item(Item::Static(var)));
                }
            }
            _ => {
                stmts.push(stmt);
                break;
            }
        }
    }

    stmts.extend(istmts);

    Ok((statics, stmts))
}
