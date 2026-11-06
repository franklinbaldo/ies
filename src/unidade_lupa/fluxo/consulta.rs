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
    // Controles de paginação devem existir como o SEI os publica.
    // A biblioteca de formulário insere os ocultos fixos na ordem correta.
    derivar_submissao(
        &CONSULTAR,
        instalacao,
        &tela,
        "",
        &[
            ("selOrgaoUnidade", ""),
            ("txtSiglaUnidade", ""),
            ("txtDescricaoUnidade", ""),
            ("hdnInfraPaginaAtual", &indice),
            ("selInfraPaginacaoSuperior", &indice),
            ("selInfraPaginacaoInferior", &indice),
        ],
    )
    .map_err(|_| FalhaDaLupa::FormularioInesperado)
}

#[cfg(test)]
mod replay_integral_go {
    use super::*;
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
    use std::path::Path;

    #[test]
    #[ignore = "wip #244: cassette integral GO exige ZIP publico conferido no CI"]
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
            let html = html_da_resposta(resposta.item)
                .unwrap_or_else(|_| panic!("HTML ilegiveI no passo {}", resposta.passo));
            let url = gravacao::url_sem_consulta(&conferida.json, resposta.item);
            let mime = gravacao::mime(resposta.item);
            let resultado = consulta(&instalacao, &url, &html, &mime, pagina);
            if resultado.is_err() {
                // Diagnostico apenas no replay: subcodigo estatico, sem
                // expor action, cookie, corpo ou URL assinada.
                let codificacao = codificacao_da_tela(&mime).expect("charset publicado");
                let indice = pagina.to_string();
                let tela = TelaServida {
                    url: &url,
                    html: &html,
                    codificacao,
                };
                let causa = derivar_submissao(
                    &CONSULTAR,
                    &instalacao,
                    &tela,
                    "",
                    &[
                        ("selOrgaoUnidade", ""),
                        ("txtSiglaUnidade", ""),
                        ("txtDescricaoUnidade", ""),
                        ("hdnInfraPaginaAtual", &indice),
                        ("selInfraPaginacaoSuperior", &indice),
                        ("selInfraPaginacaoInferior", &indice),
                    ],
                )
                .err()
                .expect("derivacao recusada");
                let dom = tl::parse(&html, tl::ParserOptions::default()).expect("dom");
                let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
                    .expect("formulario publicado");
                let mut botoes = Vec::new();
                for tag in form
                    .children()
                    .all(dom.parser())
                    .iter()
                    .filter_map(tl::Node::as_tag)
                {
                    let id = crate::html_atributos::valor_do_atributo(tag, "id")
                        .ok()
                        .flatten();
                    let name = crate::html_atributos::valor_do_atributo(tag, "name")
                        .ok()
                        .flatten();
                    if id.as_deref() == Some("btnPesquisar")
                        || name.as_deref() == Some("btnPesquisar")
                    {
                        botoes.push((
                            tag.name().as_utf8_str().to_string(),
                            crate::html_atributos::valor_do_atributo(tag, "type")
                                .ok()
                                .flatten(),
                            name.is_some(),
                            crate::html_atributos::valor_do_atributo(tag, "onclick")
                                .ok()
                                .flatten()
                                .is_some(),
                        ));
                    }
                }
                panic!(
                    "formulario passo {}: {}; botoes={:?}; onsubmit={}",
                    resposta.passo,
                    causa.subcodigo(),
                    botoes,
                    crate::html_atributos::valor_do_atributo(form, "onsubmit")
                        .ok()
                        .flatten()
                        .is_some()
                );
            }
            resultado.expect("submissao sem recusa");
        }
        assert_eq!((get, post), (1, 274), "lupa integral com 275 respostas");
    }
}
