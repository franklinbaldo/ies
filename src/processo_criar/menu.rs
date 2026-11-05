//! Link de "Iniciar Processo" publicado pelo Controle da sessão.
//! Não inferir URL, não executar JS e não seguir redirecionamento.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::instalacao::Instalacao;
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};

const ACAO: &str = "procedimento_escolher_tipo";

/// O href literal do menu; apenas o padrão relativo demonstrado pelo SEI.
/// Uma variante onclick desconhecida é recusada, não executada.
pub(crate) fn link_de_iniciar(
    instalacao: &Instalacao,
    url_controle: &str,
    html: &str,
) -> Result<String, crate::erro_html::ErroHtml> {
    use crate::erro_html::ErroHtml as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        E::valor(
            "link_de_iniciar_processo_nao_reconhecido",
            "o valor não está na forma observada",
        )
    })?;
    let parser = dom.parser();
    let mut candidatos = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no.get(parser).and_then(tl::Node::as_tag).ok_or(E::valor(
            "link_de_iniciar_processo_nao_reconhecido",
            "o valor não está na forma observada",
        ))?;
        let Some(bruto) = valor_do_atributo(tag, "href").map_err(|e| {
            E::atributo(
                "link_de_iniciar_processo_nao_reconhecido",
                "o valor não está na forma observada",
                e,
            )
        })?
        else {
            continue;
        };
        let href = match decodificar_atributo(&bruto) {
            Some(href) => href,
            None if bruto.contains(ACAO) => {
                return Err(E::valor(
                    "link_de_iniciar_processo_nao_reconhecido",
                    "o valor não está na forma observada",
                ));
            }
            None => continue,
        };
        let aponta = href.split_once('?').is_some_and(|(_, consulta)| {
            consulta
                .split('&')
                .any(|par| par == "acao=procedimento_escolher_tipo")
        });
        if aponta {
            candidatos.push(href);
        }
    }
    let [href] = candidatos.as_slice() else {
        return Err(if candidatos.is_empty() {
            E::ausente(
                "link_de_iniciar_processo_ausente",
                "a tela não tem o elemento observado",
            )
        } else {
            E::repetido(
                "link_de_iniciar_processo_repetido",
                "o elemento aparece mais de uma vez",
            )
        });
    };
    // Não seguir links de outra instalação, nem admitir parâmetros inesperados
    // de ação ou identificador de sessão.
    let consulta = href.strip_prefix("controlador.php?").ok_or(E::valor(
        "link_de_iniciar_processo_nao_reconhecido",
        "o valor não está na forma observada",
    ))?;
    if href.contains(['#', '\\', '\'']) || !href.bytes().all(|b| (33..=126).contains(&b)) {
        return Err(E::valor(
            "link_de_iniciar_processo_nao_reconhecido",
            "o valor não está na forma observada",
        ));
    }
    let (mut acoes, mut hashes) = (0, 0);
    for par in consulta.split('&') {
        let (chave, valor) = par.split_once('=').ok_or(E::valor(
            "link_de_iniciar_processo_nao_reconhecido",
            "o valor não está na forma observada",
        ))?;
        conferir_chave_de_consulta(chave).map_err(|_| {
            E::valor(
                "link_de_iniciar_processo_nao_reconhecido",
                "o valor não está na forma observada",
            )
        })?;
        conferir_valor_de_consulta(valor).map_err(|_| {
            E::valor(
                "link_de_iniciar_processo_nao_reconhecido",
                "o valor não está na forma observada",
            )
        })?;
        match chave {
            "acao" if valor == ACAO => acoes += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "acao" | "infra_hash" | "acao_ajax" => {
                return Err(E::valor(
                    "link_de_iniciar_processo_nao_reconhecido",
                    "o valor não está na forma observada",
                ));
            }
            _ => {}
        }
    }
    if acoes != 1 || hashes != 1 {
        return Err(E::valor(
            "link_de_iniciar_processo_nao_reconhecido",
            "o valor não está na forma observada",
        ));
    }
    let base = url_controle.split('?').next().ok_or(E::valor(
        "link_de_iniciar_processo_nao_reconhecido",
        "o valor não está na forma observada",
    ))?;
    let (pasta, _) = base.rsplit_once('/').ok_or(E::valor(
        "link_de_iniciar_processo_nao_reconhecido",
        "o valor não está na forma observada",
    ))?;
    let url = format!("{pasta}/{href}");
    let decomposicao = crate::url_observada::UrlHttps::interpretar(&url).ok_or(E::valor(
        "link_de_iniciar_processo_nao_reconhecido",
        "o valor não está na forma observada",
    ))?;
    if !instalacao.autoriza_controlador(decomposicao.host, decomposicao.caminho) {
        return Err(E::valor(
            "link_de_iniciar_processo_nao_reconhecido",
            "o valor não está na forma observada",
        ));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> (&'static str, Instalacao) {
        (
            "https://sei.exemplo.test/sei/controlador.php?acao=procedimento_controlar",
            Instalacao::interpretar("https://sei.exemplo.test/sei").unwrap(),
        )
    }

    fn ancora(acao: &str) -> String {
        format!(
            "<a href='controlador.php?acao={acao}&amp;infra_hash={}'>Iniciar</a>",
            "a".repeat(64)
        )
    }

    #[test]
    fn aceita_um_link_servido_e_preserva_assinatura() {
        let (url, instalacao) = base();
        let link = link_de_iniciar(&instalacao, url, &ancora(ACAO)).unwrap();
        assert!(link.contains("acao=procedimento_escolher_tipo&infra_hash="));
        assert!(!link.contains("amp;"));
    }

    #[test]
    fn recusa_ausencia_repeticao_e_acao_errada() {
        let (url, instalacao) = base();
        assert_eq!(
            link_de_iniciar(&instalacao, url, "<div/>"),
            Err(crate::erro_html::ErroHtml::ausente(
                "link_de_iniciar_processo_ausente",
                "a tela não tem o elemento observado"
            ))
        );
        let repetido = format!("{}{}", ancora(ACAO), ancora(ACAO));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &repetido),
            Err(crate::erro_html::ErroHtml::repetido(
                "link_de_iniciar_processo_repetido",
                "o elemento aparece mais de uma vez"
            ))
        );
        assert_eq!(
            link_de_iniciar(&instalacao, url, &ancora("procedimento_gerar")),
            Err(crate::erro_html::ErroHtml::ausente(
                "link_de_iniciar_processo_ausente",
                "a tela não tem o elemento observado"
            ))
        );
    }

    #[test]
    fn recusa_link_externo_ou_sem_assinatura() {
        let (url, instalacao) = base();
        let externo = format!(
            "<a href='https://outro.test/controlador.php?acao={ACAO}&amp;infra_hash={}'>Iniciar</a>",
            "a".repeat(64)
        );
        assert_eq!(
            link_de_iniciar(&instalacao, url, &externo),
            Err(crate::erro_html::ErroHtml::valor(
                "link_de_iniciar_processo_nao_reconhecido",
                "o valor não está na forma observada"
            ))
        );
        let sem_assinatura = format!("<a href='controlador.php?acao={ACAO}'>Iniciar</a>");
        assert_eq!(
            link_de_iniciar(&instalacao, url, &sem_assinatura),
            Err(crate::erro_html::ErroHtml::valor(
                "link_de_iniciar_processo_nao_reconhecido",
                "o valor não está na forma observada"
            ))
        );
    }
}
