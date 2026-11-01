//! Leitura opt-in de HTML privado previamente obtido da interface Web.
//! Nunca estabelece sessão nem executa GET, POST ou escrita no SEI.
use crate::{Cli, Documento, Domain, documento, documento_web, transporte};
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroHtmlPrivado {
    QuantidadeDeDocumentos,
    OpcoesDeDocumentoSemEvidencia,
    Numero(documento::NumeroDocumentoInvalido),
    Fonte(transporte::FalhaTransporte),
    Titulo(documento_web::ErroTituloDocumento),
    Corpo(documento_web::ErroCorpoDocumento),
    NumeroDiferenteDoSolicitado,
}

impl ErroHtmlPrivado {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::QuantidadeDeDocumentos => "html_privado_exige_um_documento",
            Self::OpcoesDeDocumentoSemEvidencia => "opcoes_de_documento_incompativeis_com_html_privado",
            Self::Numero(erro) => erro.subcodigo(),
            Self::Fonte(erro) => erro.subcodigo(),
            Self::Titulo(erro) => erro.subcodigo(),
            Self::Corpo(erro) => erro.subcodigo(),
            Self::NumeroDiferenteDoSolicitado => "numero_do_html_diferente_do_solicitado",
        }
    }
    fn status(self) -> u8 {
        match self {
            Self::QuantidadeDeDocumentos
            | Self::OpcoesDeDocumentoSemEvidencia
            | Self::Numero(_) => 2,
            _ => 1,
        }
    }
}

fn conferir_comando(
    documento: &Documento,
    unidade: Option<&str>,
) -> Result<documento::NumeroDocumento, ErroHtmlPrivado> {
    if documento.numeros.len() != 1 {
        return Err(ErroHtmlPrivado::QuantidadeDeDocumentos);
    }
    if documento.processo.is_some()
        || documento.versao.is_some()
        || documento.sem_copia
        || documento.saida.is_some()
        || documento.ocultar_inacessiveis
        || documento.com_propriedades
        || documento.acao.is_some()
        || unidade.is_some()
    {
        return Err(ErroHtmlPrivado::OpcoesDeDocumentoSemEvidencia);
    }
    documento::NumeroDocumento::novo(&documento.numeros[0]).map_err(ErroHtmlPrivado::Numero)
}

/// Seleção positiva deve verificar identidade exata, sem converter zeros.
fn selecionar(
    html: &str,
    numero: &documento::NumeroDocumento,
) -> Result<(documento_web::IdentidadeDoDocumento, String), ErroHtmlPrivado> {
    let _ = (html, numero);
    Err(ErroHtmlPrivado::Titulo(
        documento_web::ErroTituloDocumento::TituloAusente,
    ))
}

pub(super) fn tentar_ler(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(documento) = &cli.command else {
        return None;
    };
    let Some(arquivo) = documento.html_privado.as_deref() else {
        return None;
    };
    let resultado = (|| {
        let numero = conferir_comando(documento, cli.unidade.as_deref())?;
        let html = transporte::arquivo_privado(arquivo, 8_000_000)
            .map_err(ErroHtmlPrivado::Fonte)?;
        selecionar(&html, &numero)
    })();
    Some(match resultado {
        Ok((identidade, texto)) => {
            if cli.json {
                println!("{}", json!({
                    "documento": identidade, "texto": texto,
                    "escopo": "html_privado_offline",
                    "autenticacao_remota": false,
                    "versao_atual": null
                }));
            } else {
                println!("IES: documento {} ({}) observado em HTML privado local; sessão e versão remotas não comprovadas.", identidade.numero, identidade.tipo);
                println!("{texto}");
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            if cli.json {
                println!("{}", json!({
                    "codigo": if erro.status() == 2 { "entrada_invalida" } else { "fonte_invalida" },
                    "subcodigo": erro.subcodigo(),
                    "erro": "leitura de documento no HTML privado não comprovada",
                    "retentavel": false
                }));
            } else {
                eprintln!("IES: HTML privado de documento recusado [{}]", erro.subcodigo());
            }
            ExitCode::from(erro.status())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn html(numero: &str) -> String {
        format!(
            "<html><head><title>SEI/UnB - {numero} - Ofício</title></head>\
             <body><p>Texto preservado da captura.</p></body></html>"
        )
    }

    #[test]
    fn html_com_numero_exato_devolve_texto_sem_versao_inventada() {
        let numero = documento::NumeroDocumento::novo("0007").unwrap();
        let (identidade, texto) = selecionar(&html("0007"), &numero).unwrap();
        assert_eq!(identidade.numero, "0007");
        assert_eq!(identidade.tipo, "Ofício");
        assert_eq!(texto, "Texto preservado da captura.");
    }

    #[test]
    fn numero_nao_casado_nao_aceita_zero_a_esquerda_perdido() {
        let numero = documento::NumeroDocumento::novo("7").unwrap();
        assert_eq!(
            selecionar(&html("0007"), &numero),
            Err(ErroHtmlPrivado::NumeroDiferenteDoSolicitado)
        );
    }

    #[test]
    fn recusa_corpo_com_script_sem_expor_segredos() {
        let numero = documento::NumeroDocumento::novo("0007").unwrap();
        let suspeito = html("0007").replace(
            "<p>Texto preservado da captura.</p>",
            "<script>valor_interno</script><p>Texto</p>"
        );
        assert_eq!(
            selecionar(&suspeito, &numero),
            Err(ErroHtmlPrivado::Corpo(
                documento_web::ErroCorpoDocumento::CorpoComConteudoExecutavel
            ))
        );
    }

    #[test]
    fn contrato_cli_exige_um_documento_e_recusa_opcoes_sem_evidencia() {
        let cli = Cli::try_parse_from([
            "ies", "documento", "0007", "--html-privado", "/tmp/privado.html"
        ]).unwrap();
        let Domain::Documento(documento) = cli.command else { panic!("tipo diferente") };
        assert!(conferir_comando(&documento, None).is_ok());

        let cli = Cli::try_parse_from([
            "ies", "documento", "0007", "0008", "--html-privado", "/tmp/privado.html"
        ]).unwrap();
        let Domain::Documento(documento) = cli.command else { panic!("tipo diferente") };
        assert_eq!(
            conferir_comando(&documento, None),
            Err(ErroHtmlPrivado::QuantidadeDeDocumentos)
        );
        let cli = Cli::try_parse_from([
            "ies", "documento", "0007", "--html-privado", "/tmp/privado.html", "--versao", "2"
        ]).unwrap();
        let Domain::Documento(documento) = cli.command else { panic!("tipo diferente") };
        assert_eq!(
            conferir_comando(&documento, None),
            Err(ErroHtmlPrivado::OpcoesDeDocumentoSemEvidencia)
        );
    }
}
