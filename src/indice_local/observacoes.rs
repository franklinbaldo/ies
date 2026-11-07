//! Observação oportunista de unidade ativa; falha do índice não falha o GET.
use super::IndiceLocal;
use crate::instalacao::Instalacao;
use iesei::observacao::{UnidadeExibida, VisaoDoControle};

impl IndiceLocal {
    pub(crate) fn registrar_unidade_observada(
        &self,
        instalacao: &Instalacao,
        contexto: &str,
        unidade: &UnidadeExibida,
        visao: VisaoDoControle,
    ) {
        self.executar((), |banco, agora| {
            iesei::sqlite::registrar_unidade(banco, instalacao, contexto, unidade, visao, agora)
        });
    }
}
