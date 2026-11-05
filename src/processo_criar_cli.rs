//! Superfície da criação de processos (#245).
//! Nunca transforma um comando incompleto em escrita remota: a etapa Web
//! só será habilitada depois de observar e testar o cadastro servido.
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte};
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

/// Uma etapa incompleta nunca envia GET/POST com potencial de efeito.
/// Argumentos são conferidos antes de qualquer acesso ao arquivo privado.
pub(crate) fn tentar_criar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Criar {
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
    // O nível já foi restringido pelo parser clap. Não assumir hipótese
    // legal, unidade ou formulário a partir de argumentos.
    let _ = (nivel, confirmar);
    if let Err(erro) = configuracao::arquivos_de_acesso() {
        return Some(erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()));
    }
    Some(erro_fonte(
        cli.json,
        "fluxo_de_criacao_ainda_nao_validado",
        "criação indisponível: falta conferir a tela Web antes da prévia e do único POST",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrada_invalida_eh_recusada_sem_sessao() {
        assert_eq!(
            validar(" ", "Ensaio", &[]),
            Err(ErroEntradaCriacao::Tipo)
        );
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
