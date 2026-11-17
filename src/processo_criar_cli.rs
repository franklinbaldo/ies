//! Superfície da criação de processos (#245).
//! Nunca transforma um comando incompleto em escrita remota: a etapa Web
//! só será habilitada depois de observar e testar o cadastro servido.
use crate::processo_criar_fluxo::{ErroCriar, PedidoCriar, criar_com_contagem};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, configuracao, transporte};
#[path = "processo_criar_cli/apresentacao.rs"]
mod apresentacao;
use apresentacao::informar_sucesso;

/// Argumento de criação vazio ou com caractere de controle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct ArgumentoDeCriacaoInvalido;

/// `--hipotese-legal` fora da forma `ID#NOME` exata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct HipoteseLegalInvalida;

fn texto_valido(valor: &str) -> bool {
    !valor.trim().is_empty() && !valor.chars().any(char::is_control)
}

/// `--tipo`, `--especificacao` e `--interessado`: o clap recusa (saída 2).
pub(crate) fn argumento_de_criacao(valor: &str) -> Result<String, ArgumentoDeCriacaoInvalido> {
    if texto_valido(valor) {
        Ok(valor.to_owned())
    } else {
        Err(ArgumentoDeCriacaoInvalido)
    }
}

/// `--hipotese-legal ID#NOME`: identifica ID e nome observados, sem espaços nas pontas.
pub(crate) fn hipotese_legal(escolha: &str) -> Result<String, HipoteseLegalInvalida> {
    let Some((id, nome)) = escolha.split_once('#') else {
        return Err(HipoteseLegalInvalida);
    };
    if !texto_valido(id) || !texto_valido(nome) || id != id.trim() || nome != nome.trim() {
        return Err(HipoteseLegalInvalida);
    }
    Ok(escolha.to_owned())
}

/// A falha do fluxo, com os pedidos efetivamente tentados.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct FalhaNaCriacao {
    #[source]
    causa: ErroCriar,
    requisicoes: u32,
}

/// Valida os argumentos antes de acessar os arquivos privados da sessão.
/// Nenhum retry: o fluxo remoto consome cada resposta exatamente uma vez.
pub(crate) fn tentar_criar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Criar {
                tipo,
                especificacao,
                interessado,
                nivel,
                hipotese_legal,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    if hipotese_legal.is_some() && *nivel != crate::dominio::NivelDeCriacao::Restrito {
        // Regra entre dois argumentos: erro de uso do clap (saída 2).
        use clap::CommandFactory as _;
        Cli::command()
            .error(
                clap::error::ErrorKind::ArgumentConflict,
                "--hipotese-legal exige --nivel restrito",
            )
            .exit();
    }
    Some(criar(
        cli,
        tipo,
        especificacao,
        interessado,
        *nivel,
        hipotese_legal.as_deref(),
        *confirmar,
    ))
}

fn criar(
    cli: &Cli,
    tipo: &str,
    especificacao: &str,
    interessados: &[String],
    nivel: crate::dominio::NivelDeCriacao,
    hipotese_legal: Option<&str>,
    confirmar: bool,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let referencias = transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    )?;
    let pedido = PedidoCriar {
        tipo,
        especificacao,
        interessados,
        nivel,
        unidade: cli.unidade.as_deref(),
        hipotese_legal,
        confirmar,
    };
    let mut requisicoes = 0;
    let feito = criar_com_contagem(
        referencias.origem(),
        referencias.url_do_controle(),
        referencias.cookie(),
        &pedido,
        &mut EnvioHttps,
        &mut requisicoes,
    )
    .map_err(|causa| FalhaNaCriacao { causa, requisicoes })?;
    informar_sucesso(cli.json, feito, especificacao);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argumento_vazio_ou_com_controle_eh_recusado() {
        assert_eq!(argumento_de_criacao(" "), Err(ArgumentoDeCriacaoInvalido));
        assert_eq!(argumento_de_criacao("\n"), Err(ArgumentoDeCriacaoInvalido));
        assert_eq!(argumento_de_criacao("\t"), Err(ArgumentoDeCriacaoInvalido));
        assert_eq!(
            argumento_de_criacao("Início de Processo"),
            Ok("Início de Processo".to_owned())
        );
    }

    #[test]
    fn hipotese_legal_exige_identificacao_exata() {
        assert_eq!(hipotese_legal("31"), Err(HipoteseLegalInvalida));
        assert_eq!(hipotese_legal("#Base A"), Err(HipoteseLegalInvalida));
        assert_eq!(hipotese_legal("31# Base A"), Err(HipoteseLegalInvalida));
        assert_eq!(hipotese_legal("31#Base A"), Ok("31#Base A".to_owned()));
    }
}
