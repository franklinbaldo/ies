//! Filtro local das descrições de andamentos de histórico já validado.
//! Não consulta o SEI nem transforma ausência na captura em inexistência.
use crate::processo::DetalheProcesso;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroFiltroAndamentos {
    TermoVazio,
    TermoComControle,
    TermoAcimaDoLimite,
    ImplementacaoPendente,
}
impl ErroFiltroAndamentos {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::TermoVazio => "termo_de_andamento_vazio",
            Self::TermoComControle => "termo_de_andamento_contem_controle",
            Self::TermoAcimaDoLimite => "termo_de_andamento_acima_do_limite",
            Self::ImplementacaoPendente => "filtro_de_andamentos_nao_implementado",
        }
    }
}
fn validar_termo(termo: &str) -> Result<&str, ErroFiltroAndamentos> {
    let termo = termo.trim();
    if termo.is_empty() {
        return Err(ErroFiltroAndamentos::TermoVazio);
    }
    if termo.chars().any(char::is_control) {
        return Err(ErroFiltroAndamentos::TermoComControle);
    }
    if termo.chars().count() > 100 {
        return Err(ErroFiltroAndamentos::TermoAcimaDoLimite);
    }
    Ok(termo)
}

/// Devolve o total original; só é permitido filtrar depois do parser completo.
pub fn filtrar(detalhe: &mut DetalheProcesso, termo: &str) -> Result<usize, ErroFiltroAndamentos> {
    validar_termo(termo)?;
    let _ = detalhe;
    Err(ErroFiltroAndamentos::ImplementacaoPendente)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processo::{Andamento, NumeroProcesso};

    fn historico() -> DetalheProcesso {
        let mut detalhe = DetalheProcesso::novo(NumeroProcesso::novo("00012").unwrap());
        for descricao in ["Processo recebido", "Juntada de decisão", "DECISÃO assinada"] {
            detalhe.andamentos.push(Andamento {
                descricao: descricao.into(),
                ocorrido_em: None, unidade: None, usuario: None
            });
        }
        detalhe
    }

    #[test]
    fn seleciona_somente_descricoes_com_correspondencia_unicode_sem_perder_nup() {
        let mut detalhe = historico();
        assert_eq!(filtrar(&mut detalhe, " decisão "), Ok(3));
        assert_eq!(detalhe.numero.as_str(), "00012");
        assert_eq!(detalhe.andamentos.len(), 2);
        assert_eq!(detalhe.andamentos[0].descricao, "Juntada de decisão");
        assert_eq!(detalhe.andamentos[1].descricao, "DECISÃO assinada");
    }
    #[test]
    fn ausencia_de_termo_na_captura_eh_resultado_vazio_nao_inexistencia() {
        let mut detalhe = historico();
        assert_eq!(filtrar(&mut detalhe, "nunca ocorreu"), Ok(3));
        assert!(detalhe.andamentos.is_empty());
    }
    #[test]
    fn entradas_ruins_falham_por_variantes_tipadas_sem_alterar_historico() {
        let mut detalhe = historico();
        for (termo, esperado) in [
            (" ", ErroFiltroAndamentos::TermoVazio),
            ("linha\nsecreta", ErroFiltroAndamentos::TermoComControle),
        ] {
            assert_eq!(filtrar(&mut detalhe, termo), Err(esperado));
            assert_eq!(detalhe.andamentos.len(), 3);
        }
        assert_eq!(
            filtrar(&mut detalhe, &"x".repeat(101)),
            Err(ErroFiltroAndamentos::TermoAcimaDoLimite)
        );
        assert_eq!(detalhe.andamentos.len(), 3);
    }
}
