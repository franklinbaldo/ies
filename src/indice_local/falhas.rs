//! Falhas que o índice detecta e não derrubam o comando (ADR 0003): o
//! índice as guarda e a CLI as publica como `avisos`. O índice desligado
//! por configuração (`IES_SEM_INDICE`) não é falha.
use super::IndiceLocal;
use super::abertura::ErroAberturaDoIndice;
use super::validacao::RegistroRecusado;
use serde_json::{Value, json};

/// Uma falha do índice, sem caminho, URL, cookie nem hash.
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[error("{self:?}")]
#[serde(tag = "falha", content = "causa", rename_all = "snake_case")]
pub(crate) enum FalhaDoIndice {
    /// O banco falhou e o índice se desligou neste processo.
    Banco(#[source] ErroAberturaDoIndice),
    /// A entrada de uma gravação foi recusada antes de tocar no banco.
    RegistroRecusado(RegistroRecusado),
    /// A coluna do aceite da entrada por id guardava um valor fora de
    /// `{0, 1}`.
    EstadoDoAceiteForaDoConjunto(i64),
}

impl IndiceLocal {
    pub(super) fn anotar(&self, falha: FalhaDoIndice) {
        let mut falhas = self.falhas.take();
        falhas.push(falha);
        self.falhas.set(falhas);
    }

    /// Guarda a recusa da gravação, se houve; `true` se a entrada passou.
    pub(super) fn anotar_recusa(&self, resultado: Result<(), RegistroRecusado>) -> bool {
        match resultado {
            Ok(()) => true,
            Err(recusa) => {
                self.anotar(FalhaDoIndice::RegistroRecusado(recusa));
                false
            }
        }
    }

    /// As falhas detectadas até aqui, na ordem; o índice as esquece.
    pub(crate) fn avisos(&self) -> Vec<FalhaDoIndice> {
        self.falhas.take()
    }
}

/// `avisos` no objeto da saída JSON (ou em cada objeto do array da lista
/// D), omitida quando o índice não detectou falha.
pub(crate) fn anexar_avisos(valor: &mut Value, indice: &IndiceLocal) {
    let avisos = indice.avisos();
    if avisos.is_empty() {
        return;
    }
    let lista = json!(avisos);
    if let Some(objeto) = valor.as_object_mut() {
        objeto.insert("avisos".to_owned(), lista);
    } else if let Some(itens) = valor.as_array_mut() {
        for objeto in itens.iter_mut().filter_map(Value::as_object_mut) {
            objeto.insert("avisos".to_owned(), lista.clone());
        }
    }
}

/// Sem `--json`: uma linha `Aviso do índice: …` em `stderr` por falha, com
/// a mesma forma tipada do JSON.
pub(crate) fn avisar_no_texto(indice: &IndiceLocal) {
    for falha in indice.avisos() {
        eprintln!("Aviso do índice: {}", json!(falha));
    }
}
