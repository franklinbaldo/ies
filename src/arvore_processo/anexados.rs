//! Processos anexados da árvore (Refs #59; contracts/cli/processo-obter-arvore.md).
//! Forma observada na `UnB` 4.0.9 (03/11/2026): `PROCESSO_ANEXADO` com 16
//! argumentos, filho do processo, a3 `arvore_visualizar` com o id do pai e
//! o do anexado, a12 `null` (ou `"noVisitado"` quando a árvore do pai é
//! aberta a partir do anexado), a13 e a14 `null` e a15 igual a a5. Os documentos do
//! anexado não vêm nesta árvore; filho de anexado nunca foi observado e é
//! recusa.
use super::pastas::{NoDePasta, Pai};
use super::{ARGUMENTOS, DocumentoLido, ErroArvore, Literal, ProcessoNaArvore};
use std::collections::HashSet;

/// Um anexado lido, com o pai (a2) ainda não conferido.
pub(super) struct AnexadoLido {
    pai: String,
    anexado: ProcessoNaArvore,
}

pub(super) fn ler_anexado(a: &[Literal; ARGUMENTOS]) -> Result<AnexadoLido, ErroArvore> {
    let fora = ErroArvore::ProcessoAnexadoForaDaFormaObservada;
    let id = super::id(&a[1])?;
    let pai = super::id(&a[2])?;
    if !endereco_observado(super::texto(&a[3])?, &pai, &id) {
        return Err(fora);
    }
    // a12 é estado visual: `"noVisitado"` quando a árvore foi aberta a
    // partir do próprio anexado (`id_procedimento_anexado` no `ifrArvore`).
    let a12_observado = a[12] == Literal::Nulo || a[12] == Literal::Texto("noVisitado".to_owned());
    if !a12_observado || a[13..15].iter().any(|l| *l != Literal::Nulo) {
        return Err(fora);
    }
    let nup = super::texto(&a[5])?;
    if nup.is_empty() || super::texto(&a[15])? != nup {
        return Err(ErroArvore::NupDoNoDoProcessoAnexadoInconsistente);
    }
    Ok(AnexadoLido {
        pai,
        anexado: ProcessoNaArvore {
            id,
            nup: nup.to_owned(),
            tipo: super::texto(&a[6])?.to_owned(),
        },
    })
}

/// `controlador.php?…` relativo com uma `acao=arvore_visualizar`, um
/// `id_procedimento` igual ao pai e um `id_procedimento_anexado` igual ao id.
fn endereco_observado(url: &str, pai: &str, id: &str) -> bool {
    let Some(consulta) = url.strip_prefix("controlador.php?") else {
        return false;
    };
    let mut vistos = [0u8; 3];
    for par in consulta.split('&') {
        let (chave, valor) = par.split_once('=').unwrap_or((par, ""));
        let (posicao, esperado) = match chave {
            "acao" => (0, "arvore_visualizar"),
            "id_procedimento" => (1, pai),
            "id_procedimento_anexado" => (2, id),
            _ => continue,
        };
        if valor != esperado {
            return false;
        }
        vistos[posicao] += 1;
    }
    vistos == [1, 1, 1]
}

/// Confere os anexados contra o processo e os outros nós; devolve-os na
/// ordem da árvore.
pub(super) fn conferir_anexados(
    processo: &ProcessoNaArvore,
    lidos: Vec<AnexadoLido>,
    documentos: &[DocumentoLido],
    nos_de_pasta: &[NoDePasta],
) -> Result<Vec<ProcessoNaArvore>, ErroArvore> {
    let mut ids: HashSet<&str> = HashSet::new();
    let mut nups: HashSet<&str> = HashSet::new();
    for lido in &lidos {
        let a = &lido.anexado;
        if a.id == processo.id || a.nup == processo.nup {
            return Err(ErroArvore::ProcessoAnexadoIgualAoProcesso);
        }
        if !ids.insert(&a.id) || !nups.insert(&a.nup) {
            return Err(ErroArvore::ProcessoAnexadoRepetidoNaArvore);
        }
    }
    let filho_de_anexado = lidos.iter().any(|l| ids.contains(l.pai.as_str()))
        || documentos
            .iter()
            .any(|d| matches!(&d.pai, Pai::Processo(pai) if ids.contains(pai.as_str())))
        || nos_de_pasta
            .iter()
            .any(|n| matches!(n, NoDePasta::Pasta { pai, .. } if ids.contains(pai.as_str())));
    if filho_de_anexado {
        return Err(ErroArvore::NoComPaiNoProcessoAnexado);
    }
    if lidos.iter().any(|l| l.pai != processo.id) {
        return Err(ErroArvore::ProcessoAnexadoComPaiDiferenteDoProcesso);
    }
    Ok(lidos.into_iter().map(|l| l.anexado).collect())
}
