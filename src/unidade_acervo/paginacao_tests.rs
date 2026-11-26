//! Diagnóstico estrutural do paginador original, sem valores ou URLs.
use super::{HtmlServido, arvore};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_submissao,
};
use crate::instalacao::Instalacao;

fn identificador(valor: &str) -> bool {
    !valor.is_empty() && valor.len() <= 80 && valor.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub(super) fn observar_formularios(
    instalacao: &str,
    html: &HtmlServido,
    gravacao: &serde_json::Value,
    _interacao: &serde_json::Value,
    inst: &Instalacao,
    unidade: &str,
) {
    let dom = arvore(html);
    let mut candidatos = 0;
    for form in dom.elementos("form") {
        let id = form
            .atributo("id")
            .filter(|id| id.starts_with("frm") && identificador(id))
            .unwrap_or("(outro)");
        let post = form
            .atributo("method")
            .is_some_and(|m| m.eq_ignore_ascii_case("post"));
        for campo in form.elementos("input") {
            if campo.atributo("type") != Some("hidden") {
                continue;
            }
            let Some(nome) = campo
                .atributo("name")
                .filter(|n| n.starts_with("hdn") && n.ends_with("PaginaAtual") && identificador(n))
            else {
                continue;
            };
            candidatos += 1;
            let zero = campo.atributo("value") == Some("0");
            assert_eq!(id, "frmEstatisticasLista", "{instalacao}");
            assert!(post, "{instalacao}: formulario deve usar POST");
            assert_eq!(nome, "hdnInfraPaginaAtual", "{instalacao}");
            assert!(zero, "{instalacao}: indice inicial deve ser zero");
            println!("{instalacao}: form={id} post={post} campo={nome} indice_zero={zero}");
        }
    }
    assert_eq!(candidatos, 1, "{instalacao}: campos de paginacao");
    println!("{instalacao}: campos candidatos de paginacao={candidatos}");
    // Cada POST usa o HTML e o charset da resposta imediatamente anterior.
    // O gravador não guarda o corpo do POST nem a query da requisição.
    use crate::cassette::acervo::gravacao as fonte;
    let respostas: Vec<_> = fonte::respostas(gravacao)
        .into_iter()
        .filter(|r| r.operacao == "estatisticas_detalhar_unidade" && r.status == 200)
        .collect();
    let mut avancos = 0;
    let mut voltas = 0;
    for (indice, (anterior, seguinte)) in respostas.iter().zip(respostas.iter().skip(1)).enumerate()
    {
        assert_eq!(seguinte.metodo, "POST", "{instalacao}");
        let html_anterior = HtmlServido::de_teste(
            &crate::cassette::html_da_resposta(anterior.interacao).expect("HTML original"),
        );
        let lida = super::detalhe(&html_anterior).expect("detalhe anterior");
        let final_da_lista = lida
            .legenda
            .faixa
            .as_ref()
            .is_none_or(|f| f.fim == lida.legenda.total);
        let destino = if final_da_lista {
            voltas += 1;
            0
        } else {
            avancos += 1;
            indice + 1
        };
        let url = fonte::url_sem_consulta(gravacao, anterior.interacao);
        let codificacao =
            crate::pagina_do_controle::codificacao_da_tela(&fonte::mime(anterior.interacao))
                .expect("charset original");
        let tela = TelaServida {
            url: &url,
            html: &html_anterior,
            codificacao,
        };
        let post = formulario_da_pagina(inst, &tela, unidade, destino)
            .expect("derivacao do POST a partir da pagina anterior");
        let esperado = format!("hdnInfraPaginaAtual={destino}");
        assert_eq!(
            post.corpo()
                .split(|b| *b == b'&')
                .filter(|p| *p == esperado.as_bytes())
                .count(),
            1,
            "{instalacao}: destino da paginacao"
        );
    }
    let esperado = match instalacao {
        "unb" => (0, 0),
        "pen1" => (6, 1),
        "pen2" => (13, 1),
        _ => panic!("instalacao nao catalogada"),
    };
    assert_eq!((avancos, voltas), esperado, "{instalacao}");
    println!("{instalacao}: avancos_derivados={avancos} voltas_derivadas={voltas}");
}

/// Reutiliza o formulário da página atual, sem fabricar action ou hash.
/// O índice é zero-based; o corpo original não foi persistido pelo gravador.
fn formulario_da_pagina(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    pagina: usize,
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    const PAGINAR: OperacaoDeFormulario = OperacaoDeFormulario {
        acao: crate::acao_do_sei::AcaoDoSei::EstatisticasDetalharUnidade,
        formulario: "frmEstatisticasLista",
        envio: EnvioDoFormulario::SubmitPorScript,
        campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
        campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
        campos_usuario: &[],
        resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
        unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
        efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
        },
    };
    derivar_submissao(
        &PAGINAR,
        instalacao,
        tela,
        Some(unidade),
        &[("hdnInfraPaginaAtual", &pagina.to_string())],
    )
}

#[test]
fn paginador_reescreve_indice_e_preserva_ocultos_da_tela_atual() {
    use crate::formulario_web::tests::{HASH_TELA, instalacao, tela_iso};
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=estatisticas_detalhar_unidade&infra_hash={HASH_TELA}"
    );
    let html = HtmlServido::de_teste(&format!(
        "<form id='frmEstatisticasLista' method='post' action='controlador.php?acao=estatisticas_detalhar_unidade&amp;infra_hash={HASH_TELA}'>\
         <input type='hidden' name='hdnInfraPaginaAtual' value='0'>\
         <input type='hidden' name='hdnFiltro' value='a+b'>\
         <input type='submit' name='sbmOutro' value='Outro'></form>"
    ));
    let inst = instalacao();
    let post = formulario_da_pagina(&inst, &tela_iso(&url, &html), "U", 1).unwrap();
    assert_eq!(
        post.nomes_dos_campos(),
        ["hdnInfraPaginaAtual", "hdnFiltro"]
    );
    assert_eq!(post.corpo(), b"hdnInfraPaginaAtual=1&hdnFiltro=a%2Bb");
    let segunda = HtmlServido::de_teste(
        &html
            .as_str()
            .replace("value='0'", "value='1'")
            .replace("a+b", "novo"),
    );
    let post = formulario_da_pagina(&inst, &tela_iso(&url, &segunda), "U", 2).unwrap();
    assert_eq!(post.corpo(), b"hdnInfraPaginaAtual=2&hdnFiltro=novo");
    let ausente = HtmlServido::de_teste(&html.as_str().replace("hdnInfraPaginaAtual", "outro"));
    assert!(formulario_da_pagina(&inst, &tela_iso(&url, &ausente), "U", 1).is_err());
}
