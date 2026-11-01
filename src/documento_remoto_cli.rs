//! GET opt-in de documento por link assinado explicitamente fornecido.
//! Não resolve links, não autentica, não usa GET de editor e não faz POST.
use crate::{
    Cli, Documento, Domain, arquivo_texto_documento, configuracao, documento, documento_transporte,
    documento_web,
};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroDocumentoRemoto {
    Numero(documento::NumeroDocumentoInvalido),
    QuantidadeDeDocumentos,
    OpcoesNaoComprovadas,
    EfeitoAdministrativoDaLeituraNaoAutorizado,
    Configuracao(configuracao::ErroConfiguracao),
    Transporte(documento_transporte::ErroDocumentoTransporte),
    Titulo(documento_web::ErroTituloDocumento),
    Corpo(documento_web::ErroCorpoDocumento),
    NumeroDoHtmlDiferenteDoSolicitado,
    Exportacao(arquivo_texto_documento::ErroArquivoTexto),
}
impl ErroDocumentoRemoto {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::Numero(e) => e.subcodigo(),
            Self::QuantidadeDeDocumentos => "get_documento_exige_um_numero",
            Self::OpcoesNaoComprovadas => "opcoes_incompativeis_com_documento_web",
            Self::EfeitoAdministrativoDaLeituraNaoAutorizado => {
                "efeito_administrativo_da_leitura_nao_autorizado"
            }
            Self::Configuracao(e) => e.subcodigo(),
            Self::Transporte(e) => e.subcodigo(),
            Self::Titulo(e) => e.subcodigo(),
            Self::Corpo(e) => e.subcodigo(),
            Self::NumeroDoHtmlDiferenteDoSolicitado => "numero_do_html_diferente_do_solicitado",
            Self::Exportacao(e) => e.subcodigo(),
        }
    }
    fn status(self) -> u8 {
        match self {
            Self::Numero(_)
            | Self::QuantidadeDeDocumentos
            | Self::OpcoesNaoComprovadas
            | Self::EfeitoAdministrativoDaLeituraNaoAutorizado => 2,
            _ => 1,
        }
    }
}

fn conferir(
    comando: &Documento,
    unidade: Option<&str>,
) -> Result<documento::NumeroDocumento, ErroDocumentoRemoto> {
    if comando.numeros.len() != 1 {
        return Err(ErroDocumentoRemoto::QuantidadeDeDocumentos);
    }
    if comando.processo.is_some()
        || comando.versao.is_some()
        || comando.html_privado.is_some()
        || comando.ocultar_inacessiveis
        || comando.com_propriedades
        || comando.acao.is_some()
        || unidade.is_some()
    {
        return Err(ErroDocumentoRemoto::OpcoesNaoComprovadas);
    }
    if !comando.autorizar_efeito_de_leitura {
        return Err(ErroDocumentoRemoto::EfeitoAdministrativoDaLeituraNaoAutorizado);
    }
    documento::NumeroDocumento::novo(&comando.numeros[0]).map_err(ErroDocumentoRemoto::Numero)
}

fn selecionar(
    html: &str,
    numero: &documento::NumeroDocumento,
) -> Result<(documento_web::IdentidadeDoDocumento, String), ErroDocumentoRemoto> {
    let identidade =
        documento_web::identidade_do_titulo(html).map_err(ErroDocumentoRemoto::Titulo)?;
    if identidade.numero != numero.as_str() {
        return Err(ErroDocumentoRemoto::NumeroDoHtmlDiferenteDoSolicitado);
    }
    let texto = documento_web::texto_do_corpo(html).map_err(ErroDocumentoRemoto::Corpo)?;
    Ok((identidade, texto))
}

pub(super) fn tentar_ler(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(comando) = &cli.command else {
        return None;
    };
    let url_file = comando.url_documento_file.as_deref()?;
    let resultado: Result<_, ErroDocumentoRemoto> = (|| {
        let numero = conferir(comando, cli.unidade.as_deref())?;
        let arquivos =
            configuracao::arquivos_de_acesso().map_err(ErroDocumentoRemoto::Configuracao)?;
        let html =
            documento_transporte::ler_de_arquivos(url_file, &arquivos.cookie, &arquivos.origem)
                .map_err(ErroDocumentoRemoto::Transporte)?;
        let (identidade, texto) = selecionar(&html, &numero)?;
        let bytes = comando
            .saida
            .as_deref()
            .map(|dir| {
                arquivo_texto_documento::salvar(
                    &Path::new(dir).join(format!("{}.txt", identidade.numero)),
                    &texto,
                )
                .map_err(ErroDocumentoRemoto::Exportacao)
            })
            .transpose()?;
        Ok((identidade, texto, bytes))
    })();
    Some(match resultado {
        Ok((identidade, texto, bytes)) => {
            if cli.json {
                let mut modelo = json!({
                    "documento": identidade,
                    "escopo": "get_documento_assinado_opt_in",
                    "requisicoes": 1,
                    "autenticacao_remota": "nao_auditada",
                    "versao_atual": null
                });
                if let Some(tamanho) = bytes {
                    modelo["bytes"] = json!(tamanho);
                } else {
                    modelo["texto"] = json!(texto);
                }
                println!("{modelo}");
            } else if bytes.is_some() {
                println!(
                    "IES: documento {} exportado em arquivo privado; sessão não auditada.",
                    identidade.numero
                );
            } else {
                println!(
                    "IES: documento {} observado via GET opt-in; sessão não auditada.",
                    identidade.numero
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
                        "codigo": if erro.status() == 2 { "entrada_invalida" } else { "fonte_invalida" },
                        "subcodigo": erro.subcodigo(),
                        "retentavel": false,
                        "erro": "leitura Web de documento não comprovada"
                    })
                );
            } else {
                eprintln!("IES: GET de documento recusado [{}]", erro.subcodigo());
            }
            ExitCode::from(erro.status())
        }
    })
}

#[cfg(test)]
#[path = "documento_remoto_cli/tests.rs"]
mod tests;
