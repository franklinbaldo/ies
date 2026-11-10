//! RED/GREEN de `--pagina N`: plano, derivação e controles do formulário.
//! HTML sintético modelado na estrutura do Controle de PEN1 e Goiás
//! (02/11/2026), com unidade, ids, números e hashes fictícios.
use super::*;
use crate::formulario_web::CodificacaoDaTela;

pub(super) const HASH_TELA: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_FORM: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[derive(Clone, Copy)]
pub(super) struct Tab {
    pub(super) total: usize,
    pub(super) inicio: usize,
    pub(super) fim: usize,
    pub(super) atual: &'static str,
    pub(super) paginador: bool,
    pub(super) intervalo: bool,
}

impl Tab {
    pub(super) fn nova(total: usize, inicio: usize, fim: usize, atual: &'static str) -> Self {
        Self {
            total,
            inicio,
            fim,
            atual,
            paginador: fim < total || inicio > 1,
            intervalo: true,
        }
    }
}

#[derive(Clone)]
pub(super) struct Tela {
    pub(super) unidade: &'static str,
    pub(super) modo: &'static str,
    pub(super) hash_r: &'static str,
    pub(super) hash_g: &'static str,
    pub(super) rec: Tab,
    pub(super) ger: Tab,
    pub(super) extra: String,
}

impl Tela {
    pub(super) fn nova(rec: Tab, ger: Tab) -> Self {
        Self {
            unidade: "TESTE",
            modo: "R",
            hash_r: "cccc",
            hash_g: "dddd",
            rec,
            ger,
            extra: String::new(),
        }
    }

    fn tabela(nome: &str, base: usize, t: Tab) -> String {
        let legenda = if t.total == 0 {
            "0 registros:".to_owned()
        } else if t.intervalo {
            format!(
                "Processos ({} registros - {} a {}):",
                t.total, t.inicio, t.fim
            )
        } else {
            format!("{} registros:", t.total)
        };
        let linhas: String = if t.total == 0 {
            String::new()
        } else {
            (t.inicio..=t.fim).fold(String::new(), |mut linhas, p| {
                use std::fmt::Write as _;
                let id = base + p;
                write!(
                    linhas,
                    "<tr id='P{id}'><td><input type='checkbox' id='chk{nome}Item{p}' \
                     name='chk{nome}Item{p}' value='{id}'></td><td><a class='processoVisualizado' \
                     href='controlador.php?acao=procedimento_trabalhar'>NUP-{id}</a></td></tr>"
                )
                .unwrap();
                linhas
            })
        };
        let paginador = if t.paginador {
            format!(
                "<a id='lnk{nome}ProximaPaginaSuperior' href='javascript:void(0);' \
                 onclick=\"infraAcaoPaginar('+',0,'{nome}', null);\">»</a>\
                 <select id='sel{nome}PaginacaoSuperior' name='sel{nome}PaginacaoSuperior' \
                 onchange=\"infraAcaoPaginar('=',this.value,'{nome}', null);\">\
                 <option value='0' selected='selected'>1</option><option value='1'>2</option></select>"
            )
        } else {
            String::new()
        };
        format!(
            "<div id='div{nome}AreaPaginacaoSuperior'>{paginador}</div>\
             <table id='tblProcessos{nome}' class='infraTable tabelaControle'>\
             <caption class='infraCaption'>{legenda}</caption><tr><th>Processo</th></tr>{linhas}</table>"
        )
    }

    pub(super) fn html(&self) -> String {
        format!(
            "<html><a id='lnkInfraUnidade'>{}</a>\
             <form id='frmProcedimentoControlar' class='h-100' method='post' \
             action='controlador.php?acao=procedimento_controlar&acao_origem=procedimento_controlar&infra_sistema=1&infra_unidade_atual=2&infra_hash={HASH_FORM}'>\
             {}\
             <input type='hidden' id='hdnRecebidosPaginaAtual' name='hdnRecebidosPaginaAtual' value='{}'/>\
             <input type='hidden' id='hdnRecebidosHashCriterios' name='hdnRecebidosHashCriterios' value='{}'/>\
             {}\
             <input type='hidden' id='hdnGeradosPaginaAtual' name='hdnGeradosPaginaAtual' value='{}'/>\
             <input type='hidden' id='hdnGeradosHashCriterios' name='hdnGeradosHashCriterios' value='{}'/>\
             <input type='hidden' id='hdnTipoVisualizacao' name='hdnTipoVisualizacao' value='{}' />\
             <input type='hidden' id='hdnIdSigilosos' value='' />\
             <input type='text' id='txtBloco' name='txtBloco' value=''/>{}</form>\
             <input type='hidden' id='hdnInfraPrefixoCookie' name='hdnInfraPrefixoCookie' value='x'/></html>",
            self.unidade,
            Self::tabela("Recebidos", 1000, self.rec),
            self.rec.atual,
            self.hash_r,
            Self::tabela("Gerados", 5000, self.ger),
            self.ger.atual,
            self.hash_g,
            self.modo,
            self.extra,
        )
    }
}

pub(super) fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

pub(super) fn url() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar&infra_sistema=1&infra_unidade_atual=2&infra_hash={HASH_TELA}"
    )
}

pub(super) fn tela<'a>(url: &'a str, html: &'a str) -> TelaServida<'a> {
    TelaServida {
        url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    }
}

/// PEN1 (02/11/2026): 114 recebidos e 218 gerados, páginas de 100.
pub(super) fn pen1() -> Tela {
    Tela::nova(Tab::nova(114, 1, 100, "0"), Tab::nova(218, 1, 100, "0"))
}

use Categoria::{Gerados, Recebidos};

#[test]
fn pagina_ja_exibida_nao_exige_post() {
    let html = pen1().html();
    assert_eq!(planejar(&html, Recebidos, 1), Ok(PlanoDaPagina::JaExibida));
}

#[test]
fn pagina_seguinte_tem_indice_e_faixa_esperada() {
    let html = pen1().html();
    assert_eq!(
        planejar(&html, Recebidos, 2),
        Ok(PlanoDaPagina::Pedir {
            indice: 1,
            esperada: FaixaNaPagina {
                total: 114,
                inicio: 101,
                fim: 114
            }
        })
    );
    assert_eq!(
        planejar(&html, Gerados, 3),
        Ok(PlanoDaPagina::Pedir {
            indice: 2,
            esperada: FaixaNaPagina {
                total: 218,
                inicio: 201,
                fim: 218
            }
        })
    );
}

#[test]
fn pagina_alem_do_total_e_recusada_antes_de_qualquer_post() {
    let html = pen1().html();
    assert_eq!(
        planejar(&html, Recebidos, 3),
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo {
            categoria: Recebidos,
            solicitada: 3,
            paginas: 2
        })
    );
}

#[test]
fn goias_com_paginas_de_dez_calcula_a_ultima_pagina() {
    let html = Tela::nova(Tab::nova(131, 1, 10, "0"), Tab::nova(7990, 1, 10, "0")).html();
    assert_eq!(
        planejar(&html, Gerados, 799),
        Ok(PlanoDaPagina::Pedir {
            indice: 798,
            esperada: FaixaNaPagina {
                total: 7990,
                inicio: 7981,
                fim: 7990
            }
        })
    );
    assert!(matches!(
        planejar(&html, Gerados, 800),
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo { paginas: 799, .. })
    ));
    assert!(matches!(
        planejar(&html, Recebidos, 15),
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo { paginas: 14, .. })
    ));
}

#[test]
fn pagina_lembrada_pelo_servidor_usa_o_indice_publicado() {
    let html = Tela::nova(Tab::nova(114, 101, 114, "1"), Tab::nova(218, 1, 100, "0")).html();
    assert_eq!(planejar(&html, Recebidos, 2), Ok(PlanoDaPagina::JaExibida));
    assert_eq!(
        planejar(&html, Recebidos, 1),
        Ok(PlanoDaPagina::Pedir {
            indice: 0,
            esperada: FaixaNaPagina {
                total: 114,
                inicio: 1,
                fim: 100
            }
        })
    );
}

#[test]
fn indice_incompativel_com_a_faixa_e_recusado() {
    for (inicio, fim, atual) in [(1, 100, "1"), (101, 114, "0"), (51, 114, "2")] {
        let html = Tela::nova(Tab::nova(114, inicio, fim, atual), Tab::nova(0, 1, 0, "0")).html();
        assert_eq!(
            planejar(&html, Recebidos, 1),
            Err(
                ErroPaginaDoControle::PaginaAtualDaTabelaIncompativelComAFaixa {
                    categoria: Recebidos
                }
            ),
            "{inicio}-{fim}@{atual}"
        );
    }
}

#[test]
fn indice_ilegivel_e_recusado() {
    for atual in ["", "x", "-1", "1.0"] {
        let html = Tela::nova(Tab::nova(114, 1, 100, atual), Tab::nova(0, 1, 0, "0")).html();
        assert_eq!(
            planejar(&html, Recebidos, 2),
            Err(ErroPaginaDoControle::PaginaAtualDaTabelaIlegivel {
                categoria: Recebidos
            }),
            "{atual:?}"
        );
    }
}

#[test]
fn tabela_de_uma_pagina_ou_vazia_so_tem_a_pagina_um() {
    let mut sem_intervalo = Tab::nova(3, 1, 3, "0");
    sem_intervalo.intervalo = false;
    let html = Tela::nova(sem_intervalo, Tab::nova(0, 1, 0, "0")).html();
    assert_eq!(planejar(&html, Recebidos, 1), Ok(PlanoDaPagina::JaExibida));
    assert_eq!(planejar(&html, Gerados, 1), Ok(PlanoDaPagina::JaExibida));
    assert!(matches!(
        planejar(&html, Recebidos, 2),
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo { paginas: 1, .. })
    ));
    assert!(matches!(
        planejar(&html, Gerados, 2),
        Err(ErroPaginaDoControle::PaginaForaDoIntervalo { paginas: 1, .. })
    ));
}

#[test]
fn visao_detalhada_nao_tem_pagina_por_tabela() {
    let mut detalhada = pen1();
    detalhada.modo = "D";
    assert_eq!(
        planejar(&detalhada.html(), Recebidos, 2),
        Err(ErroPaginaDoControle::PaginaExigeVisaoResumida)
    );
}

#[test]
fn tabela_paginada_sem_paginador_publicado_e_recusada() {
    let mut tela = pen1();
    tela.rec.paginador = false;
    assert_eq!(
        planejar(&tela.html(), Recebidos, 2),
        Err(ErroPaginaDoControle::PaginadorDaTabelaNaoPublicado {
            categoria: Recebidos
        })
    );
}

fn corpo(submissao: &SubmissaoDerivada) -> String {
    String::from_utf8(submissao.corpo().to_vec()).unwrap()
}

#[test]
fn post_reescreve_so_a_pagina_da_tabela_pedida() {
    let (url, html) = (url(), pen1().html());
    let submissao =
        derivar_pagina(&instalacao(), &tela(&url, &html), "TESTE", Recebidos, 1).unwrap();
    let corpo = corpo(&submissao);
    assert!(corpo.contains("hdnRecebidosPaginaAtual=1&"), "{corpo}");
    assert!(corpo.contains("hdnGeradosPaginaAtual=0&"), "{corpo}");
    assert!(corpo.contains("hdnRecebidosHashCriterios=cccc&"));
    assert!(corpo.contains("selRecebidosPaginacaoSuperior=0&"));
    assert!(corpo.contains("selGeradosPaginacaoSuperior=0&"));
    assert!(
        corpo.ends_with("hdnTipoVisualizacao=R&txtBloco="),
        "{corpo}"
    );
    assert!(!corpo.contains("chk"));
    assert!(!corpo.contains("hdnInfraPrefixoCookie"));
    assert!(!corpo.contains("hdnIdSigilosos"));
    assert!(submissao.url().contains(HASH_FORM));
    assert_eq!(submissao.referer(), url);
    assert_eq!(
        submissao.efeito(),
        EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PaginaOuFiltroLembrado
        }
    );
    let texto = format!("{submissao:?}");
    assert!(!texto.contains(HASH_FORM) && !texto.contains(HASH_TELA) && !texto.contains("cccc"));
}

#[test]
fn post_de_gerados_mantem_recebidos_como_servido() {
    let (url, html) = (url(), pen1().html());
    let submissao = derivar_pagina(&instalacao(), &tela(&url, &html), "TESTE", Gerados, 2).unwrap();
    let corpo = corpo(&submissao);
    assert!(corpo.contains("hdnRecebidosPaginaAtual=0&"));
    assert!(corpo.contains("hdnGeradosPaginaAtual=2&"));
}

#[test]
fn unidade_diferente_da_solicitada_recusa_a_derivacao() {
    let (url, html) = (url(), pen1().html());
    assert_eq!(
        derivar_pagina(&instalacao(), &tela(&url, &html), "OUTRA", Recebidos, 1).err(),
        Some(ErroPaginaDoControle::Submissao(
            ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada
        ))
    );
}

fn derivar_com_extra(extra: &str) -> Result<SubmissaoDerivada, ErroPaginaDoControle> {
    let mut tela_extra = pen1();
    tela_extra.extra = extra.to_owned();
    let (url, html) = (url(), tela_extra.html());
    derivar_pagina(&instalacao(), &tela(&url, &html), "TESTE", Recebidos, 1)
}

#[test]
fn controles_fora_do_inventario_recusam_a_derivacao() {
    let casos = [
        (
            "<input type='text' name='txtOutro' value=''>",
            ErroControleDoFormulario::ControleNaoPrevistoNoFormularioDoControle {
                campo: "txtOutro".to_owned(),
            },
        ),
        (
            "<textarea name='txaNota'></textarea>",
            ErroControleDoFormulario::ControleNaoPrevistoNoFormularioDoControle {
                campo: "txaNota".to_owned(),
            },
        ),
        (
            "<input type='checkbox' name='chkRecebidosItem99' value='1' checked>",
            ErroControleDoFormulario::CaixaDeSelecaoMarcadaNoFormularioDoControle {
                campo: "chkRecebidosItem99".to_owned(),
            },
        ),
        (
            "<select name='selRecebidosPaginacaoInferior'><option value='0'>1</option></select>",
            ErroControleDoFormulario::PaginadorSemOpcaoSelecionada {
                campo: "selRecebidosPaginacaoInferior".to_owned(),
            },
        ),
        (
            "<input type='text' name='txtBloco' value=''>",
            ErroControleDoFormulario::ControleRepetidoNoFormularioDoControle {
                campo: "txtBloco".to_owned(),
            },
        ),
    ];
    for (extra, esperado) in casos {
        assert_eq!(
            derivar_com_extra(extra).err(),
            Some(ErroPaginaDoControle::Formulario(esperado)),
            "{extra}"
        );
    }
}

#[test]
fn controles_ignorados_pelo_submit_por_script_nao_recusam() {
    for extra in [
        "<input type='checkbox' name='chkGeradosItem7' value='1'>",
        "<input type='button' name='btnFechar' value='x'>",
        "<button type='submit' name='sbmQualquer'>x</button>",
        "<input type='text' name='txtDesabilitado' disabled>",
        "<input type='text' value='sem nome'>",
    ] {
        assert!(derivar_com_extra(extra).is_ok(), "{extra}");
    }
}

#[test]
fn codificacao_vem_do_charset_declarado() {
    for (mime, esperada) in [
        (
            "text/html; charset=iso-8859-1",
            Some(CodificacaoDaTela::Iso88591),
        ),
        (
            "text/html;charset=ISO-8859-1",
            Some(CodificacaoDaTela::Iso88591),
        ),
        (
            "text/html; charset=\"UTF-8\"",
            Some(CodificacaoDaTela::Utf8),
        ),
        ("text/html; charset=utf-8", Some(CodificacaoDaTela::Utf8)),
        ("text/html", None),
        ("text/html; charset=windows-1252", None),
    ] {
        assert_eq!(codificacao_da_tela(mime), esperada, "{mime}");
    }
}
