//! Correspondência entre protocolos observados; não valida máscaras nem consulta SEI.
use crate::processo::NumeroProcesso;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroResolucao {
    NaoEncontrado,
    Ambiguo,
}

impl ErroResolucao {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::NaoEncontrado => "processo_nao_encontrado",
            Self::Ambiguo => "processo_ambiguo",
        }
    }
}

fn forma(texto: &str) -> String {
    texto
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// A grafia textual exata (sem diferença de caixa) vence a equivalência
/// por pontuação. Nunca escolhe o primeiro entre múltiplos candidatos.
pub fn resolver<'a>(
    pedido: &NumeroProcesso,
    candidatos: impl IntoIterator<Item = &'a NumeroProcesso>,
) -> Result<&'a NumeroProcesso, ErroResolucao> {
    let candidatos: Vec<_> = candidatos.into_iter().collect();
    let exatos: Vec<_> = candidatos
        .iter()
        .copied()
        .filter(|n| n.as_str().to_lowercase() == pedido.as_str().to_lowercase())
        .collect();
    let encontrados = if exatos.is_empty() {
        let chave = forma(pedido.as_str());
        if chave.is_empty() {
            return Err(ErroResolucao::NaoEncontrado);
        }
        candidatos
            .into_iter()
            .filter(|n| forma(n.as_str()) == chave)
            .collect::<Vec<_>>()
    } else {
        exatos
    };
    match encontrados.as_slice() {
        [unico] => Ok(*unico),
        [] => Err(ErroResolucao::NaoEncontrado),
        _ => Err(ErroResolucao::Ambiguo),
    }
}

/// Resolve um processo já observado na caixa da unidade, mantendo seu ID interno.
/// Duas linhas do mesmo ID representam o mesmo processo, não ambiguidade.
pub fn resolver_na_caixa<'a>(
    pedido: &NumeroProcesso,
    caixa: &'a crate::dominio::Caixa,
) -> Result<&'a crate::dominio::Processo, ErroResolucao> {
    let linhas = caixa
        .recebidos
        .processos
        .iter()
        .chain(&caixa.gerados.processos);
    let linhas: Vec<_> = linhas.collect();
    let exatos: Vec<_> = linhas
        .iter()
        .copied()
        .filter(|linha| linha.numero.to_lowercase() == pedido.as_str().to_lowercase())
        .collect();
    let encontrados = if exatos.is_empty() {
        let chave = forma(pedido.as_str());
        if chave.is_empty() {
            return Err(ErroResolucao::NaoEncontrado);
        }
        linhas
            .into_iter()
            .filter(|linha| forma(&linha.numero) == chave)
            .collect::<Vec<_>>()
    } else {
        exatos
    };
    let mut distintos = Vec::new();
    for linha in encontrados {
        if !distintos
            .iter()
            .any(|anterior: &&crate::dominio::Processo| anterior.id == linha.id)
        {
            distintos.push(linha);
        }
    }
    match distintos.as_slice() {
        [unico] => Ok(*unico),
        [] => Err(ErroResolucao::NaoEncontrado),
        _ => Err(ErroResolucao::Ambiguo),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn n(v: &str) -> NumeroProcesso {
        NumeroProcesso::novo(v).unwrap()
    }

    #[test]
    fn aceita_protocolos_opacos_e_pontuacao_diferente() {
        let candidatos = [n("ABC-001"), n("550e8400-e29b-41d4-a716-446655440000")];
        assert_eq!(resolver(&n("abc001"), &candidatos).unwrap(), &candidatos[0]);
        assert_eq!(
            resolver(&n("550e8400e29b41d4a716446655440000"), &candidatos).unwrap(),
            &candidatos[1]
        );
    }

    #[test]
    fn exato_vence_colisao_de_pontuacao() {
        let candidatos = [n("AB-C"), n("A-BC")];
        assert_eq!(resolver(&n("AB-C"), &candidatos), Ok(&candidatos[0]));
        assert_eq!(
            resolver(&n("ABC"), &candidatos),
            Err(ErroResolucao::Ambiguo)
        );
    }

    #[test]
    fn nao_escolhe_arbitrariamente_e_preserva_zeros() {
        assert_eq!(
            resolver(&n("00123"), &[n("123")]),
            Err(ErroResolucao::NaoEncontrado)
        );
        assert_eq!(
            resolver(&n("ABC"), &[n("abc"), n("ABC")]),
            Err(ErroResolucao::Ambiguo)
        );
    }
    #[test]
    fn caixa_conserva_identidade_interna_e_nao_confunde_grupos() {
        use crate::dominio::{Caixa, Categoria, IdProcedimento, Processo, Tabela};
        let linha = |id: &str, numero: &str| Processo {
            id: IdProcedimento::novo(id).unwrap(),
            numero: numero.to_owned(),
            visualizado: false,
        };
        let caixa = Caixa {
            recebidos: Tabela::nova(Categoria::Recebidos, None, vec![linha("0042", "AB-C")])
                .unwrap(),
            gerados: Tabela::nova(Categoria::Gerados, None, vec![linha("0042", "AB-C")]).unwrap(),
        };
        let escolhido = resolver_na_caixa(&n("ABC"), &caixa).unwrap();
        assert_eq!(escolhido.id.as_str(), "0042");
        assert_eq!(escolhido.numero, "AB-C");
    }

    #[test]
    fn caixa_nao_escolhe_entre_ids_diferentes() {
        use crate::dominio::{Caixa, Categoria, IdProcedimento, Processo, Tabela};
        let linha = |id: &str, numero: &str| Processo {
            id: IdProcedimento::novo(id).unwrap(),
            numero: numero.to_owned(),
            visualizado: false,
        };
        let caixa = Caixa {
            recebidos: Tabela::nova(Categoria::Recebidos, None, vec![linha("42", "AB-C")]).unwrap(),
            gerados: Tabela::nova(Categoria::Gerados, None, vec![linha("43", "A-BC")]).unwrap(),
        };
        assert_eq!(
            resolver_na_caixa(&n("ABC"), &caixa),
            Err(ErroResolucao::Ambiguo)
        );
        assert_eq!(
            resolver_na_caixa(&n("AB-C"), &caixa).unwrap().id.as_str(),
            "42"
        );
        assert_eq!(
            resolver_na_caixa(&n("00123"), &caixa),
            Err(ErroResolucao::NaoEncontrado)
        );
    }
}
