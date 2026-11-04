//! Contagem compartilhada das tentativas de requisição HTTP.
//!
//! Registrar ANTES de chamar o transporte: recusas e falhas também contam.
//! Não executa rede, não classifica efeitos e não implementa retries.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContagemDeRequisicoes {
    total: usize,
}

impl ContagemDeRequisicoes {
    /// Registra a tentativa, inclusive quando o transporte vai falhar.
    pub(crate) fn registrar(&mut self) {
        self.total = self.total.saturating_add(1);
    }

    /// Contagem integral para os fluxos que publicam `usize`.
    pub(crate) fn total(self) -> usize {
        self.total
    }

    /// Projeção compatível com as saídas `u8`, sem truncamento.
    pub(crate) fn total_u8(self) -> u8 {
        u8::try_from(self.total).unwrap_or(u8::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::ContagemDeRequisicoes;

    #[test]
    fn registra_tentativas_antes_de_qualquer_resposta() {
        let mut contagem = ContagemDeRequisicoes::default();
        assert_eq!(contagem.total(), 0);
        contagem.registrar();
        contagem.registrar();
        assert_eq!(contagem.total(), 2);
        assert_eq!(contagem.total_u8(), 2);
    }

    #[test]
    fn projecao_u8_satura_sem_truncar() {
        let mut contagem = ContagemDeRequisicoes::default();
        for _ in 0..256 {
            contagem.registrar();
        }
        assert_eq!(contagem.total(), 256);
        assert_eq!(contagem.total_u8(), u8::MAX);
    }

    #[test]
    fn total_usize_nao_transborda() {
        let mut contagem = ContagemDeRequisicoes { total: usize::MAX };
        contagem.registrar();
        assert_eq!(contagem.total(), usize::MAX);
    }
}
