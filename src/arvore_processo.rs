//! Árvore do processo (`procedimento_visualizar`), lida sem regex e sem
//! executar JavaScript: só as linhas inteiras
//! `Nos[i] = new infraArvoreNo(…);` servidas no `script` inline, e, nas
//! pastas, `Pastas[k]['protocolos']` e a barra (`.acoes`) dos documentos.
//! Estrutura em docs/como-o-sei-funciona/arvore-do-processo.md; contrato em
//! contracts/cli/processo-obter-arvore.md. Tipos aceitos: `PROCESSO`,
//! `DOCUMENTO`, `PASTA` e `AGUARDE`; qualquer outro é recusa.
pub(crate) mod endereco;
mod erro;
mod literais;
mod pastas;
mod tela;

#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod tests_literais;
#[cfg(test)]
pub(crate) mod tests_pastas;
#[cfg(test)]
mod tests_pastas_integridade;
#[cfg(test)]
pub(crate) mod tests_tela;

pub(crate) use erro::ErroArvore;
use literais::{Literal, ler_barra, ler_declaracao, ler_protocolos};
pub(crate) use pastas::ler_acao_de_abrir_pastas;
use pastas::{NoDePasta, Pai, conferir_pastas};
pub(crate) use tela::{ErroTelaDoProcesso, ler_tela_do_processo};

/// O nó `PROCESSO`: id (a1), NUP (a5, igual a a15) e tipo (a6, a dica).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoNaArvore {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}

/// Um `DOCUMENTO`: id (a1), rótulo (a5), número SEI (a15, que pode vir
/// `null`) e a pasta (`PASTA<k>`) quando o pai (a2) é uma. A dica (a6) não
/// é guardada: em e-mails traz remetente e destinatários.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoNaArvore {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) numero_sei: Option<String>,
    pub(crate) pasta: Option<String>,
}

/// Uma `PASTA`: id do nó (`PASTA<k>`), rótulo (a5) e quantos protocolos
/// `Pastas[k]['protocolos']` declara.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastaNaArvore {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) total_declarado: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArvoreLida {
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) documentos: Vec<DocumentoNaArvore>,
    pub(crate) pastas: Vec<PastaNaArvore>,
    /// Alguma pasta veio fechada (com o nó `AGUARDE`): os documentos dela
    /// não estão nesta árvore.
    pub(crate) pastas_nao_carregadas: bool,
}

/// Argumentos por nó, observados na `UnB` 4.0.9 e em GO (`PROCESSO`,
/// `DOCUMENTO`) e na `UnB` (`PASTA`, `AGUARDE`).
const ARGUMENTOS: usize = 16;
const ARGUMENTOS_DA_PASTA: usize = 15;

/// Um documento lido, com o pai e, se for de pasta, a identidade da barra.
pub(super) struct DocumentoLido {
    pai: Pai,
    documento: DocumentoNaArvore,
    identidade: Option<String>,
}

/// Lê os nós da árvore e confere a identidade: o NUP do nó `PROCESSO` tem
/// de ser **igual** ao pedido (comparação opaca) e o id igual ao aberto.
/// As pastas carregadas são conferidas pela identidade dos documentos.
pub(crate) fn ler_arvore(
    html: &str,
    nup_pedido: &str,
    id_aberto: &str,
) -> Result<ArvoreLida, ErroArvore> {
    let mut processo: Option<ProcessoNaArvore> = None;
    let mut documentos: Vec<DocumentoLido> = Vec::new();
    // Índice do nó → posição em `documentos`, só para documentos de pasta.
    let mut de_pasta: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut nos_de_pasta: Vec<NoDePasta> = Vec::new();
    let mut protocolos: Vec<(usize, String)> = Vec::new();
    let mut proximo = 0usize;
    for linha in html.split('\n') {
        if let Some(declarados) = ler_protocolos(linha)? {
            protocolos.push(declarados);
            continue;
        }
        if let Some(i) = literais::indice_da_barra(linha)
            && let Some(&posicao) = de_pasta.get(&i)
        {
            let Some((_, barra)) = ler_barra(linha)? else {
                return Err(ErroArvore::BarraDeNoForaDaFormaObservada);
            };
            documentos[posicao].identidade = pastas::identidade_da_barra(&barra)?;
            continue;
        }
        let Some((indice, argumentos)) = ler_declaracao(linha)? else {
            continue;
        };
        if indice != proximo {
            return Err(ErroArvore::IndiceDeNoForaDaSequencia);
        }
        proximo += 1;
        let tipo = argumentos
            .first()
            .ok_or(ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)?;
        match texto(tipo)? {
            "PROCESSO" => {
                if processo.is_some() {
                    return Err(ErroArvore::NosDoProcessoRepetidosNaArvore);
                }
                processo = Some(ler_processo(&com_aridade(argumentos)?)?);
            }
            "DOCUMENTO" => {
                let lido = ler_documento(&com_aridade(argumentos)?)?;
                if documentos
                    .iter()
                    .any(|d| d.documento.id == lido.documento.id)
                {
                    return Err(ErroArvore::DocumentoRepetidoNaArvore);
                }
                if matches!(lido.pai, Pai::Pasta(_)) {
                    de_pasta.insert(indice, documentos.len());
                }
                documentos.push(lido);
            }
            "PASTA" | "AGUARDE" => {
                let argumentos: [Literal; ARGUMENTOS_DA_PASTA] = argumentos
                    .try_into()
                    .map_err(|_| ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)?;
                nos_de_pasta.push(pastas::ler_no_de_pasta(&argumentos)?);
            }
            _ => return Err(ErroArvore::TipoDeNoDaArvoreNaoSuportado),
        }
    }
    let processo = processo.ok_or(ErroArvore::NoDoProcessoAusenteNaArvore)?;
    if processo.nup != nup_pedido {
        return Err(ErroArvore::IdentidadeDoProcessoDiferenteDaPedida);
    }
    if processo.id != id_aberto {
        return Err(ErroArvore::IdDoProcessoNaArvoreDiferenteDoAberto);
    }
    let (pastas, pastas_nao_carregadas) =
        conferir_pastas(&processo.id, &nos_de_pasta, &protocolos, &documentos)?;
    Ok(ArvoreLida {
        processo,
        documentos: documentos.into_iter().map(|d| d.documento).collect(),
        pastas,
        pastas_nao_carregadas,
    })
}

fn com_aridade(argumentos: Vec<Literal>) -> Result<[Literal; ARGUMENTOS], ErroArvore> {
    argumentos
        .try_into()
        .map_err(|_| ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada)
}

fn texto(literal: &Literal) -> Result<&str, ErroArvore> {
    match literal {
        Literal::Texto(texto) => Ok(texto),
        _ => Err(ErroArvore::ArgumentoDeNoComTipoDiferenteDoObservado),
    }
}

fn texto_ou_nulo(literal: &Literal) -> Result<Option<&str>, ErroArvore> {
    match literal {
        Literal::Nulo => Ok(None),
        outro => texto(outro).map(Some),
    }
}

fn decimal(valor: &str) -> bool {
    !valor.is_empty() && valor.len() <= 20 && valor.bytes().all(|b| b.is_ascii_digit())
}

fn id(literal: &Literal) -> Result<String, ErroArvore> {
    let id = texto(literal)?;
    if !decimal(id) {
        return Err(ErroArvore::IdDeNoNaoDecimal);
    }
    Ok(id.to_owned())
}

fn ler_processo(a: &[Literal; ARGUMENTOS]) -> Result<ProcessoNaArvore, ErroArvore> {
    let id = id(&a[1])?;
    if texto_ou_nulo(&a[2])?.is_some() {
        return Err(ErroArvore::NoDoProcessoComPai);
    }
    let nup = texto(&a[5])?;
    if texto(&a[15])? != nup {
        return Err(ErroArvore::NupDoNoDoProcessoInconsistente);
    }
    Ok(ProcessoNaArvore {
        id,
        nup: nup.to_owned(),
        tipo: texto(&a[6])?.to_owned(),
    })
}

/// O pai (a2: o processo ou `PASTA<k>`) e o documento; a dica (a6) não é lida.
fn ler_documento(a: &[Literal; ARGUMENTOS]) -> Result<DocumentoLido, ErroArvore> {
    let id_documento = id(&a[1])?;
    let pai = pastas::pai_do_documento(texto(&a[2])?)?;
    let pasta = match &pai {
        Pai::Pasta(k) => Some(format!("PASTA{k}")),
        Pai::Processo(_) => None,
    };
    Ok(DocumentoLido {
        pai,
        documento: DocumentoNaArvore {
            id: id_documento,
            rotulo: texto(&a[5])?.to_owned(),
            numero_sei: texto_ou_nulo(&a[15])?.map(str::to_owned),
            pasta,
        },
        identidade: None,
    })
}
