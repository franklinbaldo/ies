//! Derivação do POST da lupa com a forma efetivamente servida.
//! Separado da navegação para manter cada peça menor que 20 KB.
use super::FalhaDaLupa;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;

const CONSULTAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "unidade_selecionar_envio_processo",
    formulario: "frmUnidadeLista",
    envio: EnvioDoFormulario::BotaoDeEnvio("btnPesquisar"),
    campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
    campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
    campos_usuario: &[
        "selOrgaoUnidade",
        "txtSiglaUnidade",
        "txtDescricaoUnidade",
        "selInfraPaginacaoSuperior",
        "selInfraPaginacaoInferior",
    ],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// O GO serve dois submits com id btnPesquisar, ambos sem name.
/// O navegador nao transmite botao sem name. A forma sintética mais antiga
/// serve um submit nomeado: nesse caso, seu par entra no corpo.
fn envio_da_lupa(html: &str) -> Result<EnvioDoFormulario, FalhaDaLupa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
    let mut nomeados = 0;
    let mut sem_nome = 0;
    for tag in form
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str();
        if elemento != "input" && elemento != "button" {
            continue;
        }
        let tipo = crate::html_atributos::valor_do_atributo(tag, "type")
            .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
        let padrao = if elemento == "button" {
            "submit"
        } else {
            "text"
        };
        if !tipo
            .as_deref()
            .unwrap_or(padrao)
            .eq_ignore_ascii_case("submit")
        {
            continue;
        }
        if crate::html_atributos::atributo_presente(tag, "disabled")
            .map_err(|_| FalhaDaLupa::FormularioInesperado)?
        {
            continue;
        }
        let nome = crate::html_atributos::valor_do_atributo(tag, "name")
            .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
        let id = crate::html_atributos::valor_do_atributo(tag, "id")
            .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
        if nome.as_deref() == Some("btnPesquisar") {
            nomeados += 1;
        } else if nome.is_none() && id.as_deref() == Some("btnPesquisar") {
            sem_nome += 1;
        }
    }
    match (nomeados, sem_nome) {
        (1, 0) => Ok(EnvioDoFormulario::BotaoDeEnvio("btnPesquisar")),
        // Usar a derivacao que nao inclui botao no corpo: a presenca
        // dos submits sem nome ja foi conferida na propria operacao.
        (0, 1..=2) => Ok(EnvioDoFormulario::SubmitPorScript),
        _ => Err(FalhaDaLupa::FormularioInesperado),
    }
}

pub(super) fn consulta(
    instalacao: &Instalacao,
    url: &str,
    html: &str,
    mime: &str,
    pagina: usize,
) -> Result<SubmissaoDerivada, FalhaDaLupa> {
    let codificacao = codificacao_da_tela(mime).ok_or(FalhaDaLupa::CharsetInesperado)?;
    let indice = pagina.to_string();
    let tela = TelaServida {
        url,
        html,
        codificacao,
    };
    let operacao = OperacaoDeFormulario {
        envio: envio_da_lupa(html)?,
        ..CONSULTAR
    };
    // UnB (cassette 57497bc11c4f) publica só uma página, sem seletores
    // de paginação. GO (cassette 85f61d67) publica ambos. Nunca inventar
    // campos de formulário ausentes; a identidade e o paginador da página
    // são conferidos estruturalmente antes de derivar o único POST.
    let lida = super::ler_pagina(html).map_err(FalhaDaLupa::Tela)?;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
    let mut superior = 0;
    let mut inferior = 0;
    for no in form
        .query_selector(dom.parser(), "select")
        .into_iter()
        .flatten()
    {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(FalhaDaLupa::FormularioInesperado)?;
        let nome = crate::html_atributos::valor_do_atributo(tag, "name")
            .map_err(|_| FalhaDaLupa::FormularioInesperado)?;
        match nome.as_deref() {
            Some("selInfraPaginacaoSuperior") => superior += 1,
            Some("selInfraPaginacaoInferior") => inferior += 1,
            _ => {}
        }
    }
    if superior > 1 || inferior > 1 || (lida.paginas > 1 && superior != 1) {
        return Err(FalhaDaLupa::FormularioInesperado);
    }
    let mut campos = vec![
        ("selOrgaoUnidade", ""),
        ("txtSiglaUnidade", ""),
        ("txtDescricaoUnidade", ""),
        ("hdnInfraPaginaAtual", indice.as_str()),
    ];
    if superior == 1 {
        campos.push(("selInfraPaginacaoSuperior", indice.as_str()));
    }
    if inferior == 1 {
        campos.push(("selInfraPaginacaoInferior", indice.as_str()));
    }
    derivar_submissao(&operacao, instalacao, &tela, "", &campos)
        .map_err(|_| FalhaDaLupa::FormularioInesperado)
}

#[cfg(test)]
mod replay_integral_go {
    use super::*;
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
    use std::path::Path;

    #[test]
    #[ignore = "exige ZIP integral GO em IES_ACERVO_CACHE; executado no CI"]
    fn formularios_da_lupa_integral_go_no_acervo() {
        // Observacao: cassette GO 85f61d67, 4 GET + 274 POST gravados.
        // Sem HTTP; nunca imprimir URLs assinadas nem corpo de formulario.
        let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
        let card = raiz.join("cassettes/go/unidade-destinos-no-indice-go--85f61d670bfc.md");
        let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
            || {
                Path::new(&std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE"))
                    .join(".cache/ies")
            },
            std::path::PathBuf::from,
        );
        let zip = ZipDoAcervo::ler(&cache.join("lote-5b6af062ae69d8c7.zip"))
            .expect("ZIP publico integral GO ausente");
        let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
            .expect("cassette integral GO nao confere hashes ou origem");
        assert_eq!(conferida.instalacao, "go");
        let instalacao = gravacao::instalacao(&conferida.json).expect("instalacao gravada");
        let mut get = 0;
        let mut post = 0;
        for resposta in gravacao::respostas(&conferida.json) {
            if resposta.operacao != "unidade_selecionar_envio_processo" {
                continue;
            }
            assert_eq!(resposta.status, 200, "status no passo {}", resposta.passo);
            let pagina = if resposta.metodo == "GET" {
                get += 1;
                0
            } else {
                assert_eq!(resposta.metodo, "POST");
                let atual = post;
                post += 1;
                atual
            };
            let html = html_da_resposta(resposta.interacao)
                .unwrap_or_else(|_| panic!("HTML ilegivel no passo {}", resposta.passo));
            let url = gravacao::url_sem_consulta(&conferida.json, resposta.interacao);
            let mime = gravacao::mime(resposta.interacao);
            consulta(&instalacao, &url, &html, &mime, pagina)
                .unwrap_or_else(|e| panic!("formulario no passo {}: {e:?}", resposta.passo));
        }
        assert_eq!((get, post), (1, 274), "lupa integral com 275 respostas");
    }
}
