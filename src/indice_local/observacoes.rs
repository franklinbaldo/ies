//! Gravação oportunista de fatos validados no índice privado.
use super::IndiceLocal;
use iesei::observacao::{ObservacaoDoControle, ProcessoNaLista};

impl IndiceLocal {
    pub(crate) fn registrar_observacao(
        &self,
        contexto: &str,
        observado: &ObservacaoDoControle<ProcessoNaLista>,
    ) {
        self.executar((), |tx, _| {
            iesei::sqlite::registrar(tx, contexto, observado)?;
            Ok(())
        });
    }
}
