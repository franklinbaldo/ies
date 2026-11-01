//! Leitura opt-in de HTML privado previamente obtido da interface Web.
//! Nunca estabelece sessão nem executa GET, POST ou escrita no SEI.
use crate::{
    Cli, Documento, Domain, arquivo_texto_documento, documento, documento_web, transporte,
};
use serde_json::json;
use std::path::Path;
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
    Exportacao(arquivo_texto_documento::ErroArquivoTexto),
}

impl ErroHtmlPrivado {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::QuantidadeDeDocumentos => "html_privado_exige_um_documento",
            Self::OpcoesDeDocumentoSemEvidencia => {
                "opcoes_de_documento_incompativeis_com_html_privado"
            }
            Self::Numero(erro) => erro.subcodigo(),
            Self::Fonte(erro) => erro.subcodigo(),
            Self::Titulo(erro) => erro.subcodigo(),
            Self::Corpo(erro) => erro.subcodigo(),
            Self::NumeroDiferenteDoSolicitado => "numero_do_html_diferente_do_solicitado",
            Self::Exportacao(erro) => erro.subcodigo(),
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
    let identidade = documento_web::identidade_do_titulo(html).map_err(ErroHtmlPrivado::Titulo)?;
    if identidade.numero != numero.as_str() {
        return Err(ErroHtmlPrivado::NumeroDiferenteDoSolicitado);
    }
    let texto = documento_web::texto_do_corpo(html).map_err(ErroHtmlPrivado::Corpo)?;
    Ok((identidade, texto))
}

/// Exporta o texto para nome derivado exclusivamente do DOC validado.
/// A gravação com `create_new` mantém o arquivo privado sem sobrescrever.
fn exportar(
    diretorio: &Path,
    identidade: &documento_web::IdentidadeDoDocumento,
    texto: &str,
) -> Result<usize, ErroHtmlPrivado> {
    let destino = diretorio.join(format!("{}.txt", identidade.numero));
    arquivo_texto_documento::salvar(&destino, texto).map_err(ErroHtmlPrivado::Exportacao)
}

pub(super) fn tentar_ler(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(documento) = &cli.command else {
        return None;
    };
    let arquivo = documento.html_privado.as_deref()?;
    let resultado: Result<_, ErroHtmlPrivado> = (|| {
        let numero = conferir_comando(documento, cli.unidade.as_deref())?;
        let html =
            transporte::arquivo_privado(arquivo, 8_000_000).map_err(ErroHtmlPrivado::Fonte)?;
        let (identidade, texto) = selecionar(&html, &numero)?;
        let exportado = documento
            .saida
            .as_deref()
            .map(|diretorio| exportar(Path::new(diretorio), &identidade, &texto))
            .transpose()?;
        Ok((identidade, texto, exportado))
    })();
    Some(match resultado {
        Ok((identidade, texto, exportado)) => {
            if let Some(bytes) = exportado {
                if cli.json {
                    println!(
                        "{}",
                        json!({
                            "documento": identidade, "bytes": bytes,
                            "escopo": "html_privado_texto_exportado_offline",
                            "autenticacao_remota": false,
                            "versao_atual": null
                        })
                    );
                } else {
                    println!(
                        "IES: texto privado do documento {} gravado em {}.txt sem rede.",
                        identidade.numero, identidade.numero
                    );
                }
            } else if cli.json {
                println!(
                    "{}",
                    json!({
                        "documento": identidade, "texto": texto,
                        "escopo": "html_privado_offline",
                        "autenticacao_remota": false,
                        "versao_atual": null
                    })
                );
            } else {
                println!(
                    "IES: documento {} ({}) observado em HTML privado local; sessão e versão remotas não comprovadas.",
                    identidade.numero, identidade.tipo
                );
                println!("{texto}");
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "codigo": if erro.status() == 2 {
                            "entrada_invalida"
                        } else if matches!(erro, ErroHtmlPrivado::Exportacao(_)) {
                            "arquivo_privado_invalido"
                        } else {
                            "fonte_invalida"
                        },
                        "subcodigo": erro.subcodigo(),
                        "erro": "leitura de documento no HTML privado não comprovada",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!(
                    "IES: HTML privado de documento recusado [{}]",
                    erro.subcodigo()
                );
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
    fn contrato_aceita_saida_para_html_privado_e_recusa_versao_remota() {
        let cli = Cli::try_parse_from([
            "ies",
            "documento",
            "0007",
            "--html-privado",
            "/tmp/captura.html",
            "--saida",
            "/tmp",
        ])
        .unwrap();
        let Domain::Documento(comando) = cli.command else {
            panic!("tipo diferente")
        };
        assert!(conferir_comando(&comando, None).is_ok());
        let numero = documento::NumeroDocumento::novo("0007").unwrap();
        let (identidade, texto) = selecionar(&html("0007"), &numero).unwrap();
        let inexistente = std::env::temp_dir()
            .join(format!("ies-html-diretorio-ausente-{}", std::process::id()))
            .join("nao-existe");
        assert_eq!(
            exportar(&inexistente, &identidade, &texto),
            Err(ErroHtmlPrivado::Exportacao(
                arquivo_texto_documento::ErroArquivoTexto::DiretorioAusente
            ))
        );
    }

    #[test]
    fn exportacao_preserva_texto_privado_e_recusa_sobrescrita() {
        let raiz = std::env::temp_dir().join(format!(
            "ies-html-texto-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&raiz).unwrap();
        let numero = documento::NumeroDocumento::novo("0007").unwrap();
        let (identidade, texto) = selecionar(&html("0007"), &numero).unwrap();
        assert_eq!(exportar(&raiz, &identidade, &texto), Ok(texto.len()));
        let destino = raiz.join("0007.txt");
        assert_eq!(std::fs::read_to_string(&destino).unwrap(), texto);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&destino).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert_eq!(
            exportar(&raiz, &identidade, "OUTRO"),
            Err(ErroHtmlPrivado::Exportacao(
                arquivo_texto_documento::ErroArquivoTexto::DestinoJaExiste
            ))
        );
        std::fs::remove_dir_all(raiz).unwrap();
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
            "<script>valor_interno</script><p>Texto</p>",
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
            "ies",
            "documento",
            "0007",
            "--html-privado",
            "/tmp/privado.html",
        ])
        .unwrap();
        let Domain::Documento(documento) = cli.command else {
            panic!("tipo diferente")
        };
        assert!(conferir_comando(&documento, None).is_ok());

        let cli = Cli::try_parse_from([
            "ies",
            "documento",
            "0007",
            "0008",
            "--html-privado",
            "/tmp/privado.html",
        ])
        .unwrap();
        let Domain::Documento(documento) = cli.command else {
            panic!("tipo diferente")
        };
        assert_eq!(
            conferir_comando(&documento, None),
            Err(ErroHtmlPrivado::QuantidadeDeDocumentos)
        );
        let cli = Cli::try_parse_from([
            "ies",
            "documento",
            "0007",
            "--html-privado",
            "/tmp/privado.html",
            "--versao",
            "2",
        ])
        .unwrap();
        let Domain::Documento(documento) = cli.command else {
            panic!("tipo diferente")
        };
        assert_eq!(
            conferir_comando(&documento, None),
            Err(ErroHtmlPrivado::OpcoesDeDocumentoSemEvidencia)
        );
    }
}
