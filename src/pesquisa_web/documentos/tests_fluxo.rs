//! RED/GREEN do fluxo de `ies documento buscar`: o mesmo orçamento de
//! `processo buscar` (GET + GET + POST; página N, mais um POST), com o POST
//! em documentos. Transporte injetável, telas sintéticas.
use super::RespostaDeDocumentos;
use super::tests_fixtures::{localizacao_de_documento, resultado_de_documentos};
use crate::colheita_do_controle::Colheita;
use crate::pesquisa_web::documentos::EmDocumentos;
use crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso;
use crate::pesquisa_web::fluxo::tests::{Pedido, Roteiro, controle_com_menu, cookie, get, ok};
use crate::pesquisa_web::fluxo::{BuscaRealizada, PedidoDeBusca, SessaoDaBusca, buscar_colhendo};
use crate::pesquisa_web::paginacao::ErroPaginacaoDaPesquisa;
use crate::pesquisa_web::resposta::FaixaDaPesquisa;
use crate::pesquisa_web::tests_fixtures::{
    ORIGEM, sem_resultado, tela_de_pesquisa, url_controle, url_da_tela, url_do_envio,
};
use crate::transporte::sessao::CookieDaSessao;

type Busca = Result<BuscaRealizada<RespostaDeDocumentos>, ErroBuscaDeProcesso>;

fn buscar(
    termo: &str,
    pagina: Option<u32>,
    respostas: Vec<(i32, String, Option<String>)>,
) -> (Busca, Vec<Pedido>) {
    let (_pasta, arquivo) = cookie();
    let url = url_controle();
    let sessao = SessaoDaBusca {
        url_do_controle: &url,
        origem: ORIGEM,
        cookie: &CookieDaSessao::ler(&arquivo),
    };
    let pedido = PedidoDeBusca {
        termo,
        pagina,
        unidade: None,
    };
    let mut roteiro = Roteiro::novo(respostas);
    let colheita = Colheita::nenhuma();
    let r = buscar_colhendo::<EmDocumentos, _>(&sessao, &pedido, &colheita, &mut roteiro);
    (r, roteiro.pedidos)
}

/// O POST em documentos: `D`, as duas caixas servidas marcadas e o termo.
fn corpo(termo: &str, inicio: u64, com_botao: bool) -> String {
    let botao = if com_botao {
        "&sbmPesquisar=Pesquisar"
    } else {
        ""
    };
    format!(
        "hdnInfraTipoPagina=1{botao}&rdoPesquisarEm=D\
         &chkSinDocumentosGerados=S&chkSinDocumentosRecebidos=S\
         &selOrgaoPesquisa%5B%5D=0&hdnIdUnidade=&hdnIdAssunto=&hdnIdAssinante=&hdnIdContato=\
         &txtProtocoloPesquisa={termo}\
         &hdnIdUsuarioGerador1=&hdnIdUsuarioGerador2=&hdnIdUsuarioGerador3=&hdnInicio={inicio}"
    )
}

fn post(termo: &str, inicio: u64, com_botao: bool) -> Pedido {
    Pedido {
        metodo: "POST",
        url: url_do_envio(),
        corpo: Some(corpo(termo, inicio, com_botao)),
    }
}

fn tres(termo: &str) -> Vec<Pedido> {
    vec![
        get(url_controle()),
        get(url_da_tela()),
        post(termo, 0, true),
    ]
}

#[test]
fn numero_sei_exato_vira_documento_unico_em_tres_requisicoes() {
    let (r, pedidos) = buscar(
        "0005001",
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            (
                302,
                String::new(),
                Some(localizacao_de_documento("2001", "3001")),
            ),
        ],
    );
    let busca = r.unwrap();
    assert_eq!(busca.requisicoes, 3);
    assert!(matches!(
        busca.resposta,
        RespostaDeDocumentos::DocumentoUnico { ref documento, .. } if documento == "3001"
    ));
    assert_eq!(pedidos, tres("0005001"));
}

#[test]
fn lista_e_sem_resultado_em_tres_requisicoes() {
    let (r, pedidos) = buscar(
        "00050",
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado_de_documentos("00050", 1, 10, "27")),
        ],
    );
    let RespostaDeDocumentos::Lista(lista) = r.unwrap().resposta else {
        panic!("esperada lista");
    };
    assert_eq!((lista.total_declarado, lista.documentos.len()), (27, 10));
    assert_eq!(pedidos, tres("00050"));
    let (r, pedidos) = buscar(
        "00050",
        None,
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(sem_resultado("00050")),
        ],
    );
    assert_eq!(r.unwrap().resposta, RespostaDeDocumentos::SemResultado);
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn pagina_3_envia_um_post_de_navegar_em_documentos() {
    let (r, pedidos) = buscar(
        "00050",
        Some(3),
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado_de_documentos("00050", 1, 10, "27")),
            ok(resultado_de_documentos("00050", 21, 27, "27")),
        ],
    );
    let busca = r.unwrap();
    assert_eq!(busca.requisicoes, 4);
    let RespostaDeDocumentos::Lista(lista) = busca.resposta else {
        panic!("esperada lista");
    };
    assert_eq!(
        lista.faixa,
        FaixaDaPesquisa {
            inicio: 21,
            fim: 27
        }
    );
    let mut esperados = tres("00050");
    esperados.push(post("00050", 20, false));
    assert_eq!(pedidos, esperados);
}

/// Goiás 5.0.3 (`pesquisa-paginas-go`, passo 4): a página 2 "Exibindo 11 -
/// 20 de 27.338" com 1 registro; a conferência da faixa usa a barra.
#[test]
fn pagina_seguinte_com_linhas_ausentes_confere_a_faixa_da_barra() {
    let pagina_2 = resultado_de_documentos("00050", 11, 11, "27.338")
        .replace("Exibindo 11 - 11 de", "Exibindo 11 - 20 de");
    let (r, pedidos) = buscar(
        "00050",
        Some(2),
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado_de_documentos("00050", 1, 10, "27.338")),
            ok(pagina_2),
        ],
    );
    let RespostaDeDocumentos::Lista(lista) = r.unwrap().resposta else {
        panic!("esperada lista");
    };
    assert_eq!(
        lista.faixa,
        FaixaDaPesquisa {
            inicio: 11,
            fim: 20
        }
    );
    assert_eq!(lista.documentos.len(), 1);
    assert_eq!(lista.linhas_ausentes_da_faixa, 9);
    assert_eq!(pedidos.len(), 4);
}

#[test]
fn pagina_fora_do_intervalo_e_recusada_antes_do_post_de_pagina() {
    let (r, pedidos) = buscar(
        "00050",
        Some(4),
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado_de_documentos("00050", 1, 10, "27")),
        ],
    );
    assert_eq!(
        r.unwrap_err(),
        ErroBuscaDeProcesso::Paginacao(ErroPaginacaoDaPesquisa::PaginaForaDoIntervalo {
            solicitada: 4,
            paginas: 3
        })
    );
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn documento_unico_tem_uma_pagina_so() {
    let (r, pedidos) = buscar(
        "0005001",
        Some(2),
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            (
                302,
                String::new(),
                Some(localizacao_de_documento("2001", "3001")),
            ),
        ],
    );
    assert_eq!(r.unwrap_err().subcodigo(), "pagina_fora_do_intervalo");
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn tela_sem_caixa_de_tipo_marcada_nao_envia_post() {
    let tela = tela_de_pesquisa().replace("value=\"S\" checked=\"checked\" />", "value=\"S\" />");
    let (r, pedidos) = buscar("00050", None, vec![ok(controle_com_menu()), ok(tela)]);
    assert_eq!(
        r.unwrap_err().subcodigo(),
        "nenhum_tipo_de_documento_marcado_na_tela"
    );
    assert_eq!(pedidos.len(), 2);
}

/// Última página como na PEN3 5.0.4 (`pesquisa-paginas-pen3`, passo 9):
/// a barra diz "16 resultados" e o paginador marca a página 2.
fn ultima_pagina_com_barra_sem_faixa(selecionada: &str) -> String {
    resultado_de_documentos("00050", 11, 16, "16")
        .replace("Exibindo 11 - 16 de 16", "16 resultados")
        .replace(
            "pesquisaPaginaSelecionada\">2<",
            &format!("pesquisaPaginaSelecionada\">{selecionada}<"),
        )
}

#[test]
fn ultima_pagina_com_n_resultados_vale_a_faixa_pedida_se_o_paginador_marca_a_pagina() {
    let respostas = |ultima: String| {
        vec![
            ok(controle_com_menu()),
            ok(tela_de_pesquisa()),
            ok(resultado_de_documentos("00050", 1, 10, "16")),
            ok(ultima),
        ]
    };
    let (r, pedidos) = buscar(
        "00050",
        Some(2),
        respostas(ultima_pagina_com_barra_sem_faixa("2")),
    );
    let RespostaDeDocumentos::Lista(lista) = r.unwrap().resposta else {
        panic!("esperada lista");
    };
    assert_eq!(
        lista.faixa,
        FaixaDaPesquisa {
            inicio: 11,
            fim: 16
        }
    );
    assert_eq!(lista.documentos.len(), 6);
    assert_eq!(pedidos.len(), 4);
    let (r, _) = buscar(
        "00050",
        Some(2),
        respostas(ultima_pagina_com_barra_sem_faixa("1")),
    );
    assert_eq!(
        r.unwrap_err().subcodigo(),
        "linhas_da_tabela_diferentes_da_faixa_da_barra"
    );
}
