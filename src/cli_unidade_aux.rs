//! Adaptadores da CLI para listas de unidades.
//! Extraídos de main.rs para manter o limite de 20 KB após integrações
//! concorrentes; a capacidade mantém seu próprio fluxo de domínio.
use crate::nivel_de_detalhe::AlvoDasColunas;
use crate::{Cli, ListaDaUnidade, UnidadeAcao};

/// `ies unidade recebidos|gerados`: uma lista por categoria.
pub(crate) fn listar(
    cli: &Cli,
    lista: &ListaDaUnidade,
    categoria: crate::unidade_categoria::CategoriaLista,
) -> anyhow::Result<()> {
    crate::unidade_categoria::listar_categoria(
        cli.unidade.as_deref(),
        cli.json,
        lista.saida.as_deref(),
        categoria,
        lista.pagina.map(usize::try_from).transpose()?,
    )
}

/// Exibe colunas ou encaminha sua definição, preservando o fluxo original.
pub(crate) fn colunas(
    cli: &Cli,
    definir: Option<&AlvoDasColunas>,
    confirmar: bool,
) -> anyhow::Result<()> {
    match definir {
        None => crate::unidade_colunas_cli::mostrar_colunas(cli.unidade.as_deref(), cli.json),
        Some(definir) => crate::unidade_colunas_definir_cli::definir_colunas(
            cli.unidade.as_deref(),
            cli.json,
            definir,
            confirmar,
        ),
    }
}

/// Data civil do acervo, conferida pelo clap antes de abrir a sessão.
#[derive(Debug, thiserror::Error)]
#[error("data civil inválida; use AAAA-MM-DD")]
pub(crate) struct DataDoAcervoInvalida;

pub(crate) fn data_do_acervo(
    s: &str,
) -> Result<crate::unidade_retornos::DataCivil, DataDoAcervoInvalida> {
    use crate::unidade_retornos::{DataCivil, MesAno};
    let (mes_ano, dia) = s.rsplit_once('-').ok_or(DataDoAcervoInvalida)?;
    let mes_ano = MesAno::ler_cli(mes_ano).ok_or(DataDoAcervoInvalida)?;
    if dia.len() != 2 || !dia.bytes().all(|b| b.is_ascii_digit()) {
        return Err(DataDoAcervoInvalida);
    }
    let dia = dia.parse().map_err(|_| DataDoAcervoInvalida)?;
    DataCivil::new(mes_ano.ano(), mes_ano.mes(), dia).ok_or(DataDoAcervoInvalida)
}

/// `ies unidade <ação>`: encaminha cada ação ao seu fluxo.
pub(crate) fn executar(cli: &Cli, acao: &UnidadeAcao) -> anyhow::Result<()> {
    let unidade = cli.unidade.as_deref();
    match acao {
        UnidadeAcao::Listar { atualizar } => {
            crate::unidade_pesquisar_cli::listar(cli.json, unidade, *atualizar)
        }
        UnidadeAcao::Pesquisar { termo } => {
            crate::unidade_pesquisar_cli::pesquisar(cli.json, unidade, termo)
        }
        UnidadeAcao::Atual => crate::unidade_atual_cli::mostrar_unidade_atual(unidade, cli.json),
        UnidadeAcao::Recebidos(lista) => listar(
            cli,
            lista,
            crate::unidade_categoria::CategoriaLista::Recebidos,
        ),
        UnidadeAcao::Gerados(lista) => listar(
            cli,
            lista,
            crate::unidade_categoria::CategoriaLista::Gerados,
        ),
        UnidadeAcao::Acervo { em } => crate::unidade_acervo::listar(unidade, cli.json, *em),
        UnidadeAcao::Retornos { mes } => {
            crate::unidade_retornos_cli::listar(unidade, cli.json, *mes)
        }
        UnidadeAcao::ListarProcessos { saida, pagina } => {
            crate::unidade_lista_cli::listar_processos_da_unidade(
                unidade,
                cli.json,
                saida.as_deref(),
                pagina.map(usize::try_from).transpose()?,
            )
        }
        UnidadeAcao::Colunas { definir, confirmar } => colunas(cli, definir.as_ref(), *confirmar),
        UnidadeAcao::Trocar { sigla, confirmar } => {
            crate::unidade_trocar_cli::trocar(sigla, *confirmar, unidade, cli.json)
        }
    }
}
