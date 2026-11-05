//! Link de "Iniciar Processo" publicado pelo Controle da sessão.
//! Não inferir URL, não executar JS e não seguir redirecionamento.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::instalacao::Instalacao;
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};

const ACAO: &str = "procedimento_escolher_tipo";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroMenuCriacao {
    Ausente,
    Repetido,
    NaoReconhecido,
}

impl ErroMenuCriacao {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Ausente => "link_de_iniciar_processo_ausente",
            Self::Repetido => "link_de_iniciar_processo_repetido",
            Self::NaoReconhecido => "link_de_iniciar_processo_nao_reconhecido",
        }
    }
}

/// O href literal do menu; apenas o padrão relativo demonstrado pelo SEI.
/// Uma variante onclick desconhecida é recusada, não executada.
pub(crate) fn link_de_iniciar(
    instalacao: &Instalacao,
    url_controle: &str,
    html: &str,
) -> Result<String, ErroMenuCriacao> {
    use ErroMenuCriacao as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::NaoReconhecido)?;
    let parser = dom.parser();
    let mut candidatos = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::NaoReconhecido)?;
        let Some(bruto) = valor_do_atributo(tag, "href").map_err(|_| E::NaoReconhecido)? else {
            continue;
        };
        let href = match decodificar_atributo(&bruto) {
            Some(href) => href,
            None if bruto.contains(ACAO) => return Err(E::NaoReconhecido),
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
            E::Ausente
        } else {
            E::Repetido
        });
    };
    // Não seguir links de outra instalação, nem admitir parâmetros inesperados
    // de ação ou identificador de sessão.
    let consulta = href
        .strip_prefix("controlador.php?")
        .ok_or(E::NaoReconhecido)?;
    if href.contains(['#', '\\', '\'']) || !href.bytes().all(|b| (33..=126).contains(&b)) {
        return Err(E::NaoReconhecido);
    }
    let (mut acoes, mut hashes) = (0, 0);
    for par in consulta.split('&') {
        let (chave, valor) = par.split_once('=').ok_or(E::NaoReconhecido)?;
        conferir_chave_de_consulta(chave).map_err(|_| E::NaoReconhecido)?;
        conferir_valor_de_consulta(valor).map_err(|_| E::NaoReconhecido)?;
        match chave {
            "acao" if valor == ACAO => acoes += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "acao" | "infra_hash" | "acao_ajax" => return Err(E::NaoReconhecido),
            _ => {}
        }
    }
    if acoes != 1 || hashes != 1 {
        return Err(E::NaoReconhecido);
    }
    let base = url_controle.split('?').next().ok_or(E::NaoReconhecido)?;
    let (pasta, _) = base.rsplit_once('/').ok_or(E::NaoReconhecido)?;
    let url = format!("{pasta}/{href}");
    let decomposicao =
        crate::url_observada::UrlHttps::interpretar(&url).ok_or(E::NaoReconhecido)?;
    if !instalacao.autoriza_controlador(decomposicao.host, decomposicao.caminho) {
        return Err(E::NaoReconhecido);
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
            Err(ErroMenuCriacao::Ausente)
        );
        let repetido = format!("{}{}", ancora(ACAO), ancora(ACAO));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &repetido),
            Err(ErroMenuCriacao::Repetido)
        );
        assert_eq!(
            link_de_iniciar(&instalacao, url, &ancora("procedimento_gerar")),
            Err(ErroMenuCriacao::Ausente)
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
            Err(ErroMenuCriacao::NaoReconhecido)
        );
        let sem_assinatura = format!("<a href='controlador.php?acao={ACAO}'>Iniciar</a>");
        assert_eq!(
            link_de_iniciar(&instalacao, url, &sem_assinatura),
            Err(ErroMenuCriacao::NaoReconhecido)
        );
    }
}
