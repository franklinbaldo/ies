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
pub(crate) enum Operacao {
    Atribuir,
    Retirar,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decisao {
    NadaAFazer,
    PreviaDeRetirada,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroMarcar {
    RotuloSemSeparador,
    IdDoMarcadorNaoDecimal,
    NomeDoMarcadorVazio,
    NomeDoMarcadorComControle,
    TextoDoMarcadorLongo,
    ListaDeMarcadoresParcial,
    RotuloDiferenteDoMarcadorLido,
    CatalogoDaUnidadeNaoObservado,
    RemocaoSemFormularioObservado,
    MarcadoresNaoLidos,
    UnidadeDaSessaoNaoPublicada,
    EscritaAindaFechada,
}
impl ErroMarcar {
    pub(crate) fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::RotuloSemSeparador => "identifique o marcador por id#nome",
            Self::IdDoMarcadorNaoDecimal => "o identificador do marcador deve ser decimal",
            Self::NomeDoMarcadorVazio => "o nome do marcador não pode ser vazio",
            Self::NomeDoMarcadorComControle => "o nome do marcador contém caractere de controle",
            Self::TextoDoMarcadorLongo => "o texto do marcador excede 250 caracteres",
            Self::ListaDeMarcadoresParcial => "a lista de marcadores do processo não está completa",
            Self::RotuloDiferenteDoMarcadorLido => {
                "o id do marcador tem nome diferente do solicitado"
            }
            Self::CatalogoDaUnidadeNaoObservado => {
                "o catálogo de marcadores da unidade não foi conferido"
            }
            Self::RemocaoSemFormularioObservado => "a submissão de remoção ainda não foi conferida",
            Self::MarcadoresNaoLidos => "a lista de marcadores do processo não foi publicada",
            Self::UnidadeDaSessaoNaoPublicada => "a unidade da sessão não foi publicada pela tela",
            Self::EscritaAindaFechada => {
                "a confirmação de marcador ainda não está disponível; execute sem --confirmar para ver a prévia"
            }
        }
    }
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
        Operacao::Atribuir => Err(ErroMarcar::CatalogoDaUnidadeNaoObservado),
    }
}

mod escrita;
pub(crate) use escrita::confirmar_retirada;

#[cfg(test)]
#[path = "processo_marcar/tests.rs"]
mod tests;
