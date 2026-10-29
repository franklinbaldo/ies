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
    texto.chars().filter(char::is_alphanumeric).flat_map(char::to_lowercase).collect()
}

/// A grafia textual exata (sem diferença de caixa) vence a equivalência
/// por pontuação. Nunca escolhe o primeiro entre múltiplos candidatos.
pub fn resolver<'a>(
    pedido: &NumeroProcesso,
    candidatos: impl IntoIterator<Item = &'a NumeroProcesso>,
) -> Result<&'a NumeroProcesso, ErroResolucao> {
    let candidatos: Vec<_> = candidatos.into_iter().collect();
    let exatos: Vec<_> = candidatos.iter().copied()
        .filter(|n| n.as_str().to_lowercase() == pedido.as_str().to_lowercase())
        .collect();
    let encontrados = if exatos.is_empty() {
        let chave = forma(pedido.as_str());
        if chave.is_empty() {
            return Err(ErroResolucao::NaoEncontrado);
        }
        candidatos.into_iter().filter(|n| forma(n.as_str()) == chave).collect::<Vec<_>>()
    } else {
        exatos
    };
    match encontrados.as_slice() {
        [unico] => Ok(*unico),
        [] => Err(ErroResolucao::NaoEncontrado),
        _ => Err(ErroResolucao::Ambiguo),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn n(v: &str) -> NumeroProcesso { NumeroProcesso::novo(v).unwrap() }

    #[test]
    fn aceita_protocolos_opacos_e_pontuacao_diferente() {
        let candidatos = [n("ABC-001"), n("550e8400-e29b-41d4-a716-446655440000")];
        assert_eq!(resolver(&n("abc001"), &candidatos).unwrap(), &candidatos[0]);
        assert_eq!(resolver(&n("550e8400e29b41d4a716446655440000"), &candidatos).unwrap(), &candidatos[1]);
    }

    #[test]
    fn exato_vence_colisao_de_pontuacao() {
        let candidatos = [n("AB-C"), n("A-BC")];
        assert_eq!(resolver(&n("AB-C"), &candidatos), Ok(&candidatos[0]));
        assert_eq!(resolver(&n("ABC"), &candidatos), Err(ErroResolucao::Ambiguo));
    }

    #[test]
    fn nao_escolhe_arbitrariamente_e_preserva_zeros() {
        assert_eq!(resolver(&n("00123"), &[n("123")]), Err(ErroResolucao::NaoEncontrado));
        assert_eq!(resolver(&n("ABC"), &[n("abc"), n("ABC")]), Err(ErroResolucao::Ambiguo));
    }
}
