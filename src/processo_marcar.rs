//! Decisão tipada de `ies processo marcar` (#255).
//! O módulo trabalha apenas com os marcadores que a tela Web efetivamente publicou.
//! Não executa escrita remota nem presume catálogo ausente na tela.
use crate::indice_local::IndiceLocal;
use crate::marcadores_processo::MarcadoresLidos;
use crate::processo_concluir::escrita::Contador;
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, Marcadores, PedidoDoProcesso, SessaoDoProcesso,
    obter_processo,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rotulo {
    pub(crate) id: String,
    pub(crate) nome: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operacao {
    Atribuir,
    Retirar,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decisao {
    NadaAFazer,
    /// Atribuição ausente da árvore: a prévia depende do catálogo da unidade.
    PreviaDeAtribuicao,
    PreviaDeRetirada,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroMarcar {
    RotuloSemSeparador,
    IdDoMarcadorNaoDecimal,
    NomeDoMarcadorVazio,
    NomeDoMarcadorComControle,
    TextoDoMarcadorLongo,
    ListaDeMarcadoresParcial,
    RotuloDiferenteDoMarcadorLido,
    RemocaoSemFormularioObservado,
    MarcadoresNaoLidos,
    UnidadeDaSessaoNaoPublicada,
    /// A releitura abriu outro processo ou outra unidade.
    ReleituraDeOutroProcessoOuUnidade,
    /// O botão `btnAdicionar` (ou o redirecionamento) não publica o destino do cadastro.
    DestinoDoCadastroAusente,
    /// O endereço do cadastro não é um endereço reconhecido da instalação.
    EnderecoDoCadastroNaoReconhecido,
    /// O endereço do cadastro pertence a outro processo.
    EnderecoDoCadastroDeOutroProcesso,
    /// A tela do cadastro não foi servida (redirecionada ou sessão encerrada).
    FormularioDeCadastroNaoLido,
    /// O MIME da tela não tem codificação suportada.
    CharsetDoFormularioNaoSuportado,
    /// O id do marcador não está no catálogo da unidade.
    OpcaoDoMarcadorNaoOferecida,
    /// O id existe no catálogo, mas o nome publicado é outro.
    NomeDoMarcadorDiferenteDoCatalogo,
    /// A barra da árvore não publicou o link de gerenciar marcadores.
    LinkDeMarcadoresNaoPublicado,
    /// O formulário foi derivado, mas o envio não saiu (recusa antes do POST).
    SubmissaoDoMarcadorNaoEnviada,
    /// A origem da sessão não é uma instalação reconhecida.
    OrigemNaoInterpretada,
    /// O formulário de gerenciamento não publica a ação de remoção do marcador.
    AcaoDeRemocaoAusente,
    /// A página de gerenciamento publica mais de uma ação de remoção.
    AcaoDeRemocaoAmbigua,
    /// A tela de gerenciamento não foi servida (redirecionada ou sessão encerrada).
    TelaDaRemocaoNaoLida,
}

/// Por que a releitura independente não provou o efeito do POST.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDaReleitura {
    /// O processo não pôde ser reaberto depois do POST.
    Processo(#[source] ErroProcessoObter),
    /// A releitura não traz a lista de marcadores que prova o efeito.
    Marcadores(ErroMarcar),
}
/// Falha depois de abrir o processo, com os pedidos já feitos.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDaMarcacao {
    /// Recusa da tela ou dos dados; nenhum POST de escrita saiu.
    Recusada { causa: ErroMarcar, requisicoes: u8 },
    /// O POST da retirada saiu e o efeito não foi comprovado: não repetir.
    RetiradaNaoConfirmada {
        requisicoes: u8,
        ausente_relido: Result<bool, ErroDaReleitura>,
    },
    /// O POST da atribuição saiu e o efeito não foi comprovado: não repetir.
    AtribuicaoNaoConfirmada {
        requisicoes: u8,
        presente_relido: Result<bool, ErroDaReleitura>,
    },
}

pub(crate) fn recusa(causa: ErroMarcar, requisicoes: u8) -> anyhow::Error {
    FalhaDaMarcacao::Recusada { causa, requisicoes }.into()
}

pub(crate) fn interpretar_rotulo(texto: &str) -> Result<Rotulo, ErroMarcar> {
    let (id, nome) = texto
        .split_once('#')
        .ok_or(ErroMarcar::RotuloSemSeparador)?;
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroMarcar::IdDoMarcadorNaoDecimal);
    }
    if nome.trim().is_empty() {
        return Err(ErroMarcar::NomeDoMarcadorVazio);
    }
    if nome.chars().any(char::is_control) {
        return Err(ErroMarcar::NomeDoMarcadorComControle);
    }
    Ok(Rotulo {
        id: id.to_owned(),
        nome: nome.to_owned(),
    })
}

/// O textarea observado limita o texto a 250 caracteres.
/// A representabilidade depende do charset do formulário servido.
pub(crate) fn validar_texto(texto: &str) -> Result<(), ErroMarcar> {
    if texto.chars().count() > 250 {
        return Err(ErroMarcar::TextoDoMarcadorLongo);
    }
    Ok(())
}

/// `--texto`: o clap recusa o longo (saída 2) antes da sessão.
pub(crate) fn texto_do_marcador(texto: &str) -> Result<String, ErroMarcar> {
    validar_texto(texto)?;
    Ok(texto.to_owned())
}

/// Só decide ausência se o número observado igualar o total declarado.
/// O id é a identidade da unidade; o nome protege contra reutilização.
pub(crate) fn decidir(
    lidos: &MarcadoresLidos,
    rotulo: &Rotulo,
    operacao: Operacao,
) -> Result<Decisao, ErroMarcar> {
    if lidos.itens.len() != lidos.total_declarado {
        return Err(ErroMarcar::ListaDeMarcadoresParcial);
    }
    let encontrado = lidos.itens.iter().find(|item| item.id == rotulo.id);
    if let Some(item) = encontrado {
        if item.nome != rotulo.nome {
            return Err(ErroMarcar::RotuloDiferenteDoMarcadorLido);
        }
        return match operacao {
            Operacao::Atribuir => Ok(Decisao::NadaAFazer),
            Operacao::Retirar => Ok(Decisao::PreviaDeRetirada),
        };
    }
    match operacao {
        Operacao::Retirar => Ok(Decisao::NadaAFazer),
        Operacao::Atribuir => Ok(Decisao::PreviaDeAtribuicao),
    }
}

/// Marcadores da releitura independente: prova só com a lista completa do
/// mesmo processo e da mesma unidade.
pub(crate) fn releitura_de_marcadores<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    pedido: &PedidoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    processo: (&str, &str),
) -> Result<MarcadoresLidos, ErroDaReleitura> {
    let (id, unidade) = processo;
    let novo = obter_processo(sessao, pedido, indice, rede).map_err(ErroDaReleitura::Processo)?;
    if novo.arvore.processo.id != id || novo.unidade.as_deref() != Some(unidade) {
        return Err(ErroDaReleitura::Marcadores(
            ErroMarcar::ReleituraDeOutroProcessoOuUnidade,
        ));
    }
    match novo.marcadores {
        Marcadores::Lidos(lidos) if lidos.itens.len() == lidos.total_declarado => Ok(lidos),
        Marcadores::Lidos(_) => Err(ErroDaReleitura::Marcadores(
            ErroMarcar::ListaDeMarcadoresParcial,
        )),
        Marcadores::Omitidos => Err(ErroDaReleitura::Marcadores(ErroMarcar::MarcadoresNaoLidos)),
    }
}

mod catalogo;
mod escrita;
pub(crate) use catalogo::executar_atribuicao;
pub(crate) use escrita::confirmar_retirada;

#[cfg(test)]
#[path = "processo_marcar/tests.rs"]
mod tests;
