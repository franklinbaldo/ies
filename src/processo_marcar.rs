//! Decisão tipada de `ies processo marcar` (#255).
//! O módulo trabalha apenas com os marcadores que a tela Web efetivamente publicou.
//! Não executa escrita remota nem presume catálogo ausente na tela.
use crate::marcadores_processo::MarcadoresLidos;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rotulo {
    pub(crate) id: String,
    pub(crate) nome: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operacao { Atribuir, Retirar }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decisao { NadaAFazer, PreviaDeRetirada }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroMarcar {
    RotuloSemSeparador,
    IdDoMarcadorNaoDecimal,
    NomeDoMarcadorVazio,
    NomeDoMarcadorComControle,
    ListaDeMarcadoresParcial,
    RotuloDiferenteDoMarcadorLido,
    CatalogoDaUnidadeNaoObservado,
    RemocaoSemFormularioObservado,
    MarcadoresNaoLidos,
}
impl ErroMarcar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::RotuloSemSeparador => "rotulo_do_marcador_sem_separador",
            Self::IdDoMarcadorNaoDecimal => "id_do_marcador_nao_decimal",
            Self::NomeDoMarcadorVazio => "nome_do_marcador_vazio",
            Self::NomeDoMarcadorComControle => "nome_do_marcador_com_controle",
            Self::ListaDeMarcadoresParcial => "lista_de_marcadores_parcial",
            Self::RotuloDiferenteDoMarcadorLido => "rotulo_diferente_do_marcador_lido",
            Self::CatalogoDaUnidadeNaoObservado => "catalogo_da_unidade_nao_observado",
            Self::RemocaoSemFormularioObservado => "remocao_sem_formulario_observado",
            Self::MarcadoresNaoLidos => "marcadores_nao_lidos_para_escrita",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::RotuloSemSeparador => "identifique o marcador por id#nome",
            Self::IdDoMarcadorNaoDecimal => "o identificador do marcador deve ser decimal",
            Self::NomeDoMarcadorVazio => "o nome do marcador não pode ser vazio",
            Self::NomeDoMarcadorComControle => "o nome do marcador contém caractere de controle",
            Self::ListaDeMarcadoresParcial => "a lista de marcadores do processo não está completa",
            Self::RotuloDiferenteDoMarcadorLido => "o id do marcador tem nome diferente do solicitado",
            Self::CatalogoDaUnidadeNaoObservado => "o catálogo de marcadores da unidade não foi conferido",
            Self::RemocaoSemFormularioObservado => "a submissão de remoção ainda não foi conferida",
            Self::MarcadoresNaoLidos => "a lista de marcadores do processo não foi publicada",
        }
    }
}
pub(crate) fn interpretar_rotulo(_texto: &str) -> Result<Rotulo, ErroMarcar> {
    Err(ErroMarcar::RotuloSemSeparador) // RED: shim até implementação
}
pub(crate) fn decidir(
    _lidos: &MarcadoresLidos, _rotulo: &Rotulo, _operacao: Operacao
) -> Result<Decisao, ErroMarcar> {
    Err(ErroMarcar::MarcadoresNaoLidos) // RED: shim até implementação
}

#[cfg(test)]
#[path = "processo_marcar/tests.rs"]
mod tests;
