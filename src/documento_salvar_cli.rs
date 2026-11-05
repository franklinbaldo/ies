//! Prévia segura: nunca executa confirmação Web. O par local é
//! `DOC.vN.html` (a foto) e `DOC.v(N+1).html` (a edição), #207.
use super::{Cli, Documento, DocumentoAcao, Domain, erro_fonte};
use crate::documento_versoes::VersaoDocumento;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::{
    configuracao, copia_documento, copia_listagem, documento, documento_versoes, transporte,
};
use serde_json::json;
use std::process::ExitCode;

/// O índice e a instalação da configuração, só para ler o SHA-256 das
/// fotos registradas; sem configuração, nenhuma foto está registrada.
struct RegistroDasFotos {
    acesso: Option<(IndiceLocal, Instalacao)>,
    documento: String,
}

impl RegistroDasFotos {
    fn da_configuracao(documento: &documento::NumeroDocumento) -> Self {
        let acesso = configuracao::arquivos_de_acesso()
            .ok()
            .and_then(|arquivos| {
                let referencias = transporte::sessao::ReferenciasDaSessao::ler(
                    &arquivos.url_controle,
                    &arquivos.origem,
                )
                .ok()?;
                let instalacao = Instalacao::interpretar(referencias.origem())?;
                Some((IndiceLocal::da_configuracao(&arquivos), instalacao))
            });
        Self {
            acesso,
            documento: documento.as_str().to_owned(),
        }
    }

    fn sha(&self, versao: VersaoDocumento) -> Option<String> {
        let (indice, instalacao) = self.acesso.as_ref()?;
        indice.foto(instalacao, &self.documento, versao)
    }
}

fn mostrar_listagem_copias(documento: &documento::NumeroDocumento, json_mode: bool) -> ExitCode {
    let Ok(pasta) = std::env::current_dir() else {
        return erro_fonte(
            json_mode,
            "leitura_diretorio_copias_falhou",
            "diretório de trabalho não pode ser lido",
        );
    };
    let registro = RegistroDasFotos::da_configuracao(documento);
    match copia_listagem::listar(&pasta, documento, &|v| registro.sha(v)) {
        Ok(copias) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "copias": copias,
                        "escopo": "versoes_locais_verificadas",
                        "versao_remota_confirmada": false,
                        "escrita_remota": false
                    })
                );
            } else {
                println!(
                    "IES: {} versão(ões) local(is) verificadas; nenhuma versão remota confirmada.",
                    copias.len()
                );
                for copia in &copias {
                    println!(
                        "  {} v{}: {}",
                        copia.documento,
                        copia.versao,
                        if copia.foto {
                            "foto registrada"
                        } else {
                            "arquivo local não registrado"
                        }
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => erro_fonte(
            json_mode,
            erro.subcodigo(),
            "listagem local de cópias não foi verificada",
        ),
    }
}

pub(super) fn tentar_salvar_documento(cli: &Cli) -> Option<ExitCode> {
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
    let documento = match documento::NumeroDocumento::novo(numero) {
        Ok(documento) => documento,
        Err(erro) => {
            return Some(erro_fonte(
                cli.json,
                erro.subcodigo(),
                "número SEI inválido",
            ));
        }
    };
    if *listar_copias {
        return Some(mostrar_listagem_copias(&documento, cli.json));
    }
    let versao = match versao
        .map(documento_versoes::VersaoDocumento::nova)
        .transpose()
    {
        Ok(versao) => versao,
        Err(erro) => return Some(erro_fonte(cli.json, erro.subcodigo(), "versão inválida")),
    };
    let registro = RegistroDasFotos::da_configuracao(&documento);
    let previa =
        copia_documento::previa_local(&std::env::current_dir().ok()?, &documento, versao, &|v| {
            registro.sha(v)
        });
    match previa {
        Ok(dados) if !confirmar => {
            if cli.json {
                println!("{}", json!({"previa": dados, "confirmado": false}));
            } else {
                println!(
                    "IES: {} v{} editada sobre a foto v{} ({} → {} bytes; sem escrita no SEI)",
                    dados.documento,
                    dados.versao_editada,
                    dados.versao_base,
                    dados.bytes_base,
                    dados.bytes_editada,
                );
            }
            Some(ExitCode::SUCCESS)
        }
        Ok(_) => Some(erro_fonte(
            cli.json,
            "escrita_remota_nao_validada",
            "confirmação recusada: faltam identidade e concorrência Web verificadas",
        )),
        Err(erro) => Some(erro_fonte(
            cli.json,
            erro.subcodigo(),
            "cópia local não pode ser conferida",
        )),
    }
}

pub(super) fn validar_documentos(numeros: &[String], json_mode: bool) -> Option<ExitCode> {
    for numero in numeros {
        if let Err(erro) = documento::NumeroDocumento::novo(numero) {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "entrada_invalida",
                        "subcodigo": erro.subcodigo(),
                        "erro": "número SEI de documento inválido",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!(
                    "IES: número SEI de documento inválido [{}]",
                    erro.subcodigo()
                );
            }
            return Some(ExitCode::from(2));
        }
    }
    None
}

pub(super) fn tentar_validar_versao(cli: &Cli) -> Option<ExitCode> {
    if let Domain::Documento(Documento {
        acao: None,
        versao: Some(versao),
        ..
    }) = &cli.command
        && let Err(erro) = documento_versoes::VersaoDocumento::nova(*versao)
    {
        return Some(erro_fonte(
            cli.json,
            erro.subcodigo(),
            "versão do documento inválida",
        ));
    }
    None
}

#[cfg(test)]
mod testes_listagem {
    use super::*;
    use clap::Parser;

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
