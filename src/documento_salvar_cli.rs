//! Prévia offline e confirmação nos editores de treinamento observados.
//! O par local é
//! `DOC.vN.html` (a foto) e `DOC.v(N+1).html` (a edição), #207. Offline:
//! não lê índice, configuração nem sessão, e não verifica a base.
use super::{Cli, Documento, DocumentoAcao, Domain};
use crate::{copia_documento, copia_listagem, documento, documento_versoes};
use serde_json::json;

#[path = "documento_salvar_cli/remoto.rs"]
mod remoto;

/// HTML opaco: somente finais de linha são canônicos no transporte.
/// A cópia original em disco não é modificada.
fn html_para_envio(html: &str) -> String {
    html.replace("\r\r\n", "\n")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn mostrar_listagem_copias(
    documento: &documento::NumeroDocumento,
    json_mode: bool,
) -> anyhow::Result<()> {
    let pasta = std::env::current_dir()?;
    {
        let copias = copia_listagem::listar(&pasta, documento)?;
        {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "copias": copias,
                        "escopo": "arquivos_locais",
                        "versao_remota_confirmada": false,
                        "escrita_remota": false
                    })
                );
            } else {
                println!(
                    "IES: {} arquivo(s) local(is); nenhuma versão conferida com o SEI.",
                    copias.len()
                );
                for copia in &copias {
                    println!(
                        "  {} v{}: {} bytes",
                        copia.documento, copia.versao, copia.bytes
                    );
                }
            }
        }
    }
    Ok(())
}

pub(super) fn tentar_salvar_documento(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Salvar {
                numero,
                versao,
                confirmar,
                listar_copias,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let documento = numero.clone();
    if *listar_copias {
        return Some(mostrar_listagem_copias(&documento, cli.json));
    }
    // A faixa (1..) é do clap.
    let versao = versao.and_then(|n| documento_versoes::VersaoDocumento::nova(n).ok());
    let pasta = match std::env::current_dir() {
        Ok(pasta) => pasta,
        Err(erro) => return Some(Err(erro.into())),
    };
    let previa = copia_documento::previa_local(&pasta, &documento, versao);
    match previa {
        Ok(dados) if !confirmar => {
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "previa": dados,
                        "confirmado": false,
                        "quebras_lf_canonicas_no_envio": true,
                        "aviso_assinaturas": "editar pode remover assinaturas; a oferta do editor ainda não foi verificada"
                    })
                );
            } else {
                println!(
                    "IES: {} v{} editada sobre a v{} local ({} → {} bytes; base não verificada; sem escrita no SEI)",
                    dados.documento,
                    dados.versao_editada,
                    dados.versao_base,
                    dados.bytes_base,
                    dados.bytes_editada,
                );
                println!(
                    "Diferenças locais (não comprovam alteração remota):\n{}",
                    dados.diff_humano
                );
                println!(
                    "A confirmação preservará o HTML opaco, normalizando apenas as quebras de linha para LF."
                );
                println!(
                    "Aviso: editar pode remover assinaturas; a oferta de edição do SEI ainda não foi verificada."
                );
            }
            Some(Ok(()))
        }
        Ok(dados) => Some(remoto::confirmar(cli, &documento, &dados)),
        Err(erro) => Some(Err(erro.into())),
    }
}

#[cfg(test)]
mod testes_listagem {
    use super::*;
    use clap::Parser;

    #[test]
    fn somente_lf_e_canonizado_sem_parser_ou_reescrita_estrutural() {
        for (bruto, esperado) in [
            ("<p>A\r\r\n<b>B", "<p>A\n<b>B"),
            (
                "<table><tr><td>1<br>\r\n2</td></tr></table>",
                "<table><tr><td>1<br>\n2</td></tr></table>",
            ),
            (
                "<ul><li>Um</li></ul>\r<p>Dois",
                "<ul><li>Um</li></ul>\n<p>Dois",
            ),
            ("a\r\r\nb\r\nc\rd", "a\nb\nc\nd"),
            ("<p>&nbsp;</p>\né 🔎", "<p>&nbsp;</p>\né 🔎"),
        ] {
            assert_eq!(html_para_envio(bruto), esperado);
        }
    }

    #[test]
    fn nao_oferece_bypass_que_permita_cr_no_transporte() {
        assert!(
            Cli::try_parse_from(["ies", "documento", "salvar", "0007", "--sem-normalizar"])
                .is_err()
        );
    }

    #[test]
    fn opcoes_de_listagem_local_nao_permitem_confirmacao_nem_versao_especifica() {
        assert!(
            Cli::try_parse_from(["ies", "documento", "salvar", "0007", "--listar-copias",]).is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "ies",
                "documento",
                "salvar",
                "0007",
                "--listar-copias",
                "--versao",
                "2",
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "ies",
                "documento",
                "salvar",
                "0007",
                "--listar-copias",
                "--confirmar",
            ])
            .is_err()
        );
    }
}
