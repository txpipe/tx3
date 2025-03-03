use convert_case::{Case, Casing};
use std::{path::Path, rc::Rc};
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_core;
use swc_ecma_ast::*;

pub fn generate(protocol: tx3_lang::Protocol, dest_path: &Path) {
    let mut module_items = Vec::new();

    module_items.push(swc_ecma_quote::quote!(
        "import { resolveProtoTx, Tx } from 'tx3';" as ModuleItem
    ));

    for tx_def in protocol.txs() {
        let tx_name = tx_def.name.as_str();
        let type_name = format!("{}Params", tx_name.to_case(Case::Pascal));
        let fn_name = tx_name.to_case(Case::Camel);
        let ir_const_name = format!("{}_IR", tx_name.to_case(Case::Constant));

        let type_members = tx_def
            .parameters
            .parameters
            .iter()
            .map(|field| {
                let field_name = field.name.as_str().to_case(Case::Camel);

                TsTypeElement::TsPropertySignature(TsPropertySignature {
                    span: DUMMY_SP,
                    readonly: false,
                    key: Box::new(Expr::Ident(Ident::new_no_ctxt(field_name.into(), DUMMY_SP))),
                    computed: false,
                    optional: false,
                    type_ann: Box::new(TsTypeAnn {
                        span: DUMMY_SP,
                        type_ann: ts_type_for_field(&field.r#type).into(),
                    })
                    .into(),
                })
            })
            .collect();

        module_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
            decl: Decl::TsTypeAlias(Box::new(TsTypeAliasDecl {
                span: DUMMY_SP,
                declare: false,
                id: Ident::new_no_ctxt(type_name.clone().into(), DUMMY_SP),
                type_params: None,
                type_ann: Box::new(TsType::TsTypeLit(TsTypeLit {
                    span: DUMMY_SP,
                    members: type_members,
                })),
            })),
            span: DUMMY_SP,
        })));

        // Generate IR constant
        let prototx = protocol.new_tx(&tx_def.name).unwrap();
        let ir_bytes_hex = hex::encode(prototx.ir_bytes());

        let ir_define_stmt = swc_ecma_quote::quote!(
            "export const $ir_const_name = { bytecode: $ir_bytes, encoding: 'hex', version: 'v1alpha1' };" as ModuleItem,
            ir_const_name = Ident::new_no_ctxt(ir_const_name.clone().into(), DUMMY_SP),
            ir_bytes: Expr = ir_bytes_hex.into(),
        );

        module_items.push(ir_define_stmt);

        let resolve_stmt = swc_ecma_quote::quote!(
            "return await resolveProtoTx({
                ir: $ir_const_name,
                args,
            });" as Stmt,
            ir_const_name = Ident::new_no_ctxt(ir_const_name.clone().into(), DUMMY_SP),
        );

        // Generate function
        let fn_decl = FnDecl {
            ident: Ident::new_no_ctxt(fn_name.into(), DUMMY_SP),
            declare: false,
            function: Box::new(Function {
                params: vec![Param {
                    span: DUMMY_SP,
                    decorators: vec![],
                    pat: Pat::Ident(BindingIdent {
                        id: Ident::new_no_ctxt("args".into(), DUMMY_SP),
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: DUMMY_SP,
                            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: DUMMY_SP,
                                type_name: TsEntityName::Ident(Ident::new_no_ctxt(
                                    type_name.into(),
                                    DUMMY_SP,
                                )),
                                type_params: None,
                            })),
                        })),
                    }),
                }],
                decorators: vec![],
                span: DUMMY_SP,
                body: Some(BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![resolve_stmt],
                    ctxt: SyntaxContext::empty(),
                }),
                is_generator: false,
                is_async: true,
                type_params: None,
                return_type: Some(Box::new(TsTypeAnn {
                    span: DUMMY_SP,
                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                        span: DUMMY_SP,
                        type_name: TsEntityName::Ident(Ident::new_no_ctxt(
                            "Promise<Tx>".into(),
                            DUMMY_SP,
                        )),
                        type_params: None,
                    })),
                })),
                ctxt: SyntaxContext::empty(),
            }),
        };

        module_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
            decl: Decl::Fn(fn_decl),
            span: DUMMY_SP,
        })));
    }

    let module = Module {
        span: DUMMY_SP,
        body: module_items,
        shebang: None,
    };

    // Use swc_ecma_codegen to generate the final TypeScript code
    let mut buf = vec![];

    let writer = swc_ecma_codegen::text_writer::JsWriter::new(
        Rc::new(swc_common::SourceMap::default()),
        "\n",
        &mut buf,
        None,
    );

    let config = swc_ecma_codegen::Config::default();

    let mut emitter = swc_ecma_codegen::Emitter {
        cfg: config,
        comments: None,
        cm: Rc::new(swc_common::SourceMap::default()),
        wr: writer,
    };

    emitter.emit_module(&module).unwrap();

    std::fs::write(dest_path.join("transactions.ts"), buf)
        .expect("Failed to write TypeScript output");
}

fn ts_type_for_field(ty: &tx3_lang::ast::Type) -> TsType {
    match ty {
        tx3_lang::ast::Type::Int => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsNumberKeyword,
        }),
        tx3_lang::ast::Type::Address => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),
        tx3_lang::ast::Type::Bool => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        }),
        tx3_lang::ast::Type::Bytes => TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident::new_no_ctxt("Uint8Array".into(), DUMMY_SP)),
            type_params: None,
        }),
        // Add other type mappings as needed
        _ => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsUnknownKeyword,
        }),
    }
}
