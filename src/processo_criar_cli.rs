//! Superfície da criação de processos (#245).
//! Nunca transforma um comando incompleto em escrita remota: a etapa Web
//! só será habilitada depois de observar e testar o cadastro servido.
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, transporte};
use crate::processo_criar_fluxo::{PedidoCriar, ErroCriar, criar};
use crate::visao_do_controle::EnvioHttps;
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroEntradaCriacao {
    Tipo,
    Especificacao,
    Interessado,
}

impl ErroEntradaCriacao {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::Tipo => "tipo_de_processo_invalido",
            Self::Especificacao => "especificacao_de_processo_invalida",
            Self::Interessado => "interessado_de_processo_invalido",
        }
    }
}

fn texto_valido(valor: &str) -> bool {
    !valor.trim().is_empty() && !valor.chars().any(char::is_control)
}

fn validar(
    tipo: &str,
    especificacao: &str,
    interessados: &[String],
) -> Result<(), ErroEntradaCriacao> {
    if !texto_valido(tipo) {
        return Err(ErroEntradaCriacao::Tipo);
    }
    if !texto_valido(especificacao) {
        return Err(ErroEntradaCriacao::Especificacao);
    }
    if interessados.iter().any(|v| !texto_valido(v)) {
        return Err(ErroEntradaCriacao::Interessado);
    }
    Ok(())
}

fn uso_invalido(json_mode: bool, erro: ErroEntradaCriacao) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": erro.subcodigo(),
                "erro": "argumento de criação de processo vazio ou com caractere de controle",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: argumento de criação inválido [{}]", erro.subcodigo());
    }
    ExitCode::from(2)
}

fn erro_do_fluxo(json_mode: bool, erro: ErroCriar) -> ExitCode {
    if json_mode {
        println!("{}", json!({
            "codigo": if erro.saida() == 3 { "efeito_incerto" } else { "fonte_invalida" },
            "subcodigo": erro.subcodigo(),
            "erro": if erro.saida() == 3 {
                "o envio pode ter criado o processo; confira os gerados antes de repetir"
            } else {
                "a tela do SEI não permitiu concluir o fluxo de criação com segurança"
            },
            "retentavel": false
        }));
    } else {
        eprintln!("IES: criação não confirmada [{}]", erro.subcodigo());
        if erro.saida() == 3 {
            eprintln!("IES: confira a caixa Gerados antes de tentar novamente.");
        }
    }
    ExitCode::from(erro.saida())
}

/// Valida os argumentos antes de acessar os arquivos privados da sessão.
/// Nenhum retry: o fluxo remoto consome cada resposta exatamente uma vez.
pub(crate) fn tentar_criar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Criar {
                tipo,
                especificacao,
                interessado,
                nivel,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    if let Err(erro) = validar(tipo, especificacao, interessado) {
        return Some(uso_invalido(cli.json, erro));
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return Some(erro_fonte(cli.json, erro.subcodigo(), erro.mensagem())),
    };
    let referencias = match transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    ) {
        Ok(referencias) => referencias,
        Err(erro) => return Some(erro_fonte(cli.json, erro.subcodigo(), erro.mensagem())),
    };
    let pedido = PedidoCriar {
        tipo,
        especificacao,
        interessados: interessado,
        nivel,
        unidade: cli.unidade.as_deref(),
        confirmar: *confirmar,
    };
    let resultado = criar(
        referencias.origem(),
        referencias.url_do_controle(),
        referencias.cookie(),
        &pedido,
        &mut EnvioHttps,
    );
    Some(match resultado {
        Ok(feito) => {
            if cli.json {
                println!("{}", json!({
                    "tipo": feito.tipo,
                    "especificacao": especificacao,
                    "assunto": feito.assunto,
                    "nivel": feito.nivel,
                    "unidade": feito.unidade,
                    "confirmado": feito.confirmado,
                    "nup": feito.nup,
                    "origem": "web",
                    "requisicoes": feito.requisicoes,
                }));
            } else {
                println!(
                    "IES: {} processo «{}» na unidade {} ({} requisições).",
                    if feito.confirmado { "criado" } else { "prévia de criação de" },
                    feito.tipo, feito.unidade, feito.requisicoes
                );
                println!("Especificação: {especificacao}");
                println!("Assunto sugerido: {}", feito.assunto);
                println!("Acesso: {}", feito.nivel);
                if let Some(nup) = feito.nup {
                    println!("NUP criado e verificado: {nup}");
                } else {
                    println!("Para criar, repita com --confirmar.");
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => erro_do_fluxo(cli.json, erro),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrada_invalida_eh_recusada_sem_sessao() {
        assert_eq!(validar(" ", "Ensaio", &[]), Err(ErroEntradaCriacao::Tipo));
        assert_eq!(
            validar("Tipo", "\n", &[]),
            Err(ErroEntradaCriacao::Especificacao)
        );
        assert_eq!(
            validar("Tipo", "Ensaio", &["\t".to_owned()]),
            Err(ErroEntradaCriacao::Interessado)
        );
    }

    #[test]
    fn entrada_valida_nao_implica_autorizacao_de_escrita() {
        assert_eq!(
            validar("Início de Processo", "Ensaio controlado", &[]),
            Ok(())
        );
    }
}
