//! Leitura offline do 302 do POST da pesquisa rápida. A URL gravada
//! do POST fornece a unidade e o sistema, como no fluxo de processo NUP.
use super::{Leitor, Registro, Resultado};
use crate::cassette::acervo::gravacao;
use crate::erro_publico::Rotulo as _;
use crate::pesquisa_web::redirecionamento::{OrigemDoTrabalho, conferir_trabalho};

pub(super) fn replay(card: &str, instalacao: &str, json: &serde_json::Value) -> Vec<Registro> {
    let origem = gravacao::instalacao(json);
    gravacao::respostas(json)
        .into_iter()
        .filter(|r| (r.operacao, r.metodo, r.status) == ("protocolo_pesquisa_rapida", "POST", 302))
        .map(|r| {
            let url_do_post = r.interacao["response"]["url"].as_str();
            let resultado = match (origem.as_ref(), url_do_post) {
                (Some(origem), Some(url)) => match conferir_trabalho(
                    gravacao::localizacao(r.interacao),
                    url,
                    origem,
                    OrigemDoTrabalho::PesquisaRapida,
                ) {
                    Ok(_) => Resultado::Lido,
                    Err(erro) => Resultado::Recusado(erro.rotulo()),
                },
                (None, _) => Resultado::NaoAplicavel("instalacao_da_cassette_nao_identificada"),
                (_, None) => Resultado::NaoAplicavel("url_do_post_nao_gravada"),
            };
            Registro {
                leitor: Leitor::PesquisaRapida,
                instalacao: instalacao.to_owned(),
                card: card.to_owned(),
                passo: r.passo,
                resultado,
                forma: None,
                total_declarado: None,
                faixa: None,
                linhas_ausentes: 0,
            }
        })
        .collect()
}
