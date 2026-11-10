//! Pastas da árvore (fatia 3 de #59; contracts/cli/processo-obter-pastas.md).
//! Forma observada na `UnB` 4.0.9: `PASTA` e `AGUARDE` com 15 argumentos,
//! `Pastas[k]['protocolos']` com os `id_rel_protocolo_protocolo` da pasta e
//! a ação `ABRIR_PASTAS` na barra do processo. A pasta carregada é conferida
//! pela **identidade** de cada documento (o `id_rel_protocolo_protocolo` da
//! barra dele), nunca pela contagem.
use super::endereco::conferir_endereco_relativo;
use super::literais::ler_chamada;
use super::{
    Barra, DocumentoLido, ErroArvore, Literal, PastaNaArvore, VerificacaoDaPasta,
    tela::EnderecoDaArvore,
};
use crate::instalacao::Instalacao;
use std::collections::{HashMap, HashSet};

/// O pai de um documento: o processo (id decimal) ou `PASTA<k>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Pai {
    Processo(String),
    Pasta(usize),
}

/// Um nó `PASTA` ou `AGUARDE` lido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum NoDePasta {
    Pasta {
        k: usize,
        pai: String,
        rotulo: String,
    },
    Aguarde {
        k: usize,
    },
}

/// `<prefixo><k>` com `k` decimal de até 6 dígitos.
fn numero_depois(valor: &str, prefixo: &str) -> Option<usize> {
    let k = valor.strip_prefix(prefixo)?;
    if k.is_empty() || k.len() > 6 || !k.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    k.parse().ok()
}

pub(super) fn pai_do_documento(pai: &str) -> Result<Pai, ErroArvore> {
    if super::decimal(pai) {
        return Ok(Pai::Processo(pai.to_owned()));
    }
    numero_depois(pai, "PASTA")
        .map(Pai::Pasta)
        .ok_or(ErroArvore::IdDeNoNaoDecimal)
}

/// `PASTA`: a1 `PASTA<k>`, a2 pai, a3 `javascript:abrirFecharPasta('PASTA<k>');`,
/// a4 `null`, a5 rótulo. `AGUARDE`: a1 `AGUARDE<k>`, a2 `PASTA<k>`, a3 `null`.
pub(super) fn ler_no_de_pasta(a: &[Literal; 15]) -> Result<NoDePasta, ErroArvore> {
    let texto = super::texto;
    if texto(&a[0])? == "AGUARDE" {
        let fora = ErroArvore::AguardeForaDaFormaObservada;
        let k = numero_depois(texto(&a[1])?, "AGUARDE").ok_or(fora)?;
        if numero_depois(texto(&a[2])?, "PASTA") != Some(k) || a[3] != Literal::Nulo {
            return Err(fora);
        }
        return Ok(NoDePasta::Aguarde { k });
    }
    let fora = ErroArvore::PastaForaDaFormaObservada;
    let k = numero_depois(texto(&a[1])?, "PASTA").ok_or(fora)?;
    if texto(&a[3])? != format!("javascript:abrirFecharPasta('PASTA{k}');") || a[4] != Literal::Nulo
    {
        return Err(fora);
    }
    Ok(NoDePasta::Pasta {
        k,
        pai: super::id(&a[2])?,
        rotulo: texto(&a[5])?.to_owned(),
    })
}

const PARAMETRO: &str = "id_rel_protocolo_protocolo=";

/// O `id_rel_protocolo_protocolo` dos links da barra: `None` se nenhum link
/// o traz; valores diferentes são recusa.
pub(super) fn identidade_da_barra(barra: &str) -> Result<Option<String>, ErroArvore> {
    let mut achado: Option<&str> = None;
    for (posicao, _) in barra.match_indices(PARAMETRO) {
        // Só o parâmetro inteiro: depois de `?` ou `&` (cru ou `&amp;`).
        if !matches!(barra[..posicao].bytes().last(), Some(b'?' | b'&' | b';')) {
            continue;
        }
        let resto = &barra[posicao + PARAMETRO.len()..];
        let fim = resto
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(resto.len());
        let valor = &resto[..fim];
        if !super::decimal(valor) {
            return Err(ErroArvore::BarraDeNoForaDaFormaObservada);
        }
        match achado {
            Some(anterior) if anterior != valor => {
                return Err(ErroArvore::IdentidadesDivergentesNaBarraDoDocumento);
            }
            _ => achado = Some(valor),
        }
    }
    Ok(achado.map(str::to_owned))
}

/// `'<p1>,<p2>,…'`: decimais, não vazia, sem repetição.
fn protocolos_declarados(texto: &str) -> Result<Vec<&str>, ErroArvore> {
    let fora = ErroArvore::ProtocolosDaPastaForaDaFormaObservada;
    let lista: Vec<&str> = texto.split(',').collect();
    let unicos: HashSet<&str> = lista.iter().copied().collect();
    if !lista.iter().all(|p| super::decimal(p)) || unicos.len() != lista.len() {
        return Err(fora);
    }
    Ok(lista)
}

/// Confere nós de pasta, declarações e documentos. Devolve as pastas na
/// ordem da árvore e se alguma ficou fechada (`AGUARDE`).
pub(super) fn conferir_pastas(
    processo: &str,
    nos: &[NoDePasta],
    protocolos: &[(usize, String)],
    documentos: &[DocumentoLido],
) -> Result<(Vec<PastaNaArvore>, bool), ErroArvore> {
    let mut pastas: Vec<(usize, &str)> = Vec::new();
    for no in nos {
        if let NoDePasta::Pasta { k, pai, rotulo } = no {
            if pastas.iter().any(|(j, _)| j == k) {
                return Err(ErroArvore::PastaRepetidaNaArvore);
            }
            if pai != processo {
                return Err(ErroArvore::PastaComPaiDiferenteDoProcesso);
            }
            pastas.push((*k, rotulo));
        }
    }
    let existe = |k: usize| pastas.iter().any(|(j, _)| *j == k);
    let mut fechadas: HashSet<usize> = HashSet::new();
    for no in nos {
        if let NoDePasta::Aguarde { k } = no {
            if !existe(*k) {
                return Err(ErroArvore::AguardeComPaiDiferenteDePasta);
            }
            if !fechadas.insert(*k) {
                return Err(ErroArvore::AguardeForaDaFormaObservada);
            }
        }
    }
    let mut declarados: HashMap<usize, Vec<&str>> = HashMap::new();
    for (k, texto) in protocolos {
        let lista = protocolos_declarados(texto)?;
        if declarados.insert(*k, lista).is_some() {
            return Err(ErroArvore::ProtocolosDaPastaDeclaradosMaisDeUmaVez);
        }
    }
    if declarados.keys().any(|k| !existe(*k)) {
        return Err(ErroArvore::ProtocolosDeclaradosParaPastaAusente);
    }
    if pastas.iter().any(|(k, _)| !declarados.contains_key(k)) {
        return Err(ErroArvore::PastaSemProtocolosDeclarados);
    }
    for d in documentos {
        let valido = match &d.pai {
            Pai::Processo(id) => id == processo,
            Pai::Pasta(k) => existe(*k),
        };
        if !valido {
            return Err(ErroArvore::DocumentoComPaiDiferenteDoProcesso);
        }
    }
    let mut lidas = Vec::with_capacity(pastas.len());
    for (k, rotulo) in &pastas {
        let da_pasta: Vec<&DocumentoLido> = documentos
            .iter()
            .filter(|d| d.pai == Pai::Pasta(*k))
            .collect();
        let (verificacao, por_identidade, por_contagem) = if fechadas.contains(k) {
            if !da_pasta.is_empty() {
                return Err(ErroArvore::PastaCarregadaPelaMetade);
            }
            (None, 0, 0)
        } else {
            let (v, i, c) = conferir_pasta(&da_pasta, &declarados[k])?;
            (Some(v), i, c)
        };
        lidas.push(PastaNaArvore {
            id: format!("PASTA{k}"),
            rotulo: (*rotulo).to_owned(),
            total_declarado: declarados[k].len(),
            verificacao,
            conferidos_por_identidade: por_identidade,
            conferidos_por_contagem: por_contagem,
        });
    }
    Ok((lidas, !fechadas.is_empty()))
}

/// Identidade por documento, o resto pela contagem: cada identidade achada
/// está na declaração e aparece uma vez, e os declarados sem documento
/// identificado são tantos quantos os documentos sem identidade. Ids de
/// documento únicos na árvore já foram conferidos na leitura.
fn conferir_pasta(
    documentos: &[&DocumentoLido],
    declarados: &[&str],
) -> Result<(VerificacaoDaPasta, usize, usize), ErroArvore> {
    let mut vistos: HashSet<&str> = HashSet::new();
    let mut sem_identidade = 0usize;
    for d in documentos {
        match &d.barra {
            Barra::NaoLida => return Err(ErroArvore::DocumentoDaPastaSemIdentidadeNaBarra),
            Barra::SemIdentidade => sem_identidade += 1,
            Barra::Identidade(identidade) => {
                let identidade = identidade.as_str();
                if !declarados.contains(&identidade) {
                    return Err(ErroArvore::DocumentoDaPastaNaoDeclarado);
                }
                if !vistos.insert(identidade) {
                    return Err(ErroArvore::ProtocoloRepetidoNaPasta);
                }
            }
        }
    }
    if declarados.len() - vistos.len() != sem_identidade {
        return Err(if sem_identidade == 0 {
            ErroArvore::ProtocoloDeclaradoAusenteDaPasta
        } else {
            ErroArvore::ContagemDaPastaDiferenteDaDeclarada
        });
    }
    let verificacao = match (vistos.len(), sem_identidade) {
        (_, 0) => VerificacaoDaPasta::PorIdentidade,
        (0, _) => VerificacaoDaPasta::PorContagem,
        _ => VerificacaoDaPasta::Mista,
    };
    Ok((verificacao, vistos.len(), sem_identidade))
}

const ACAO_DE_ABRIR: &str = "new infraArvoreAcao(\"ABRIR_PASTAS\",";

/// "Abrir todas as Pastas" publicada pela árvore: a única linha
/// `NosAcoes[j] = new infraArvoreAcao("ABRIR_PASTAS","AP<id>","<id>","<URL>",null,"…","…",true);`,
/// com a URL conferida. `Ok(None)` se a árvore não a publica.
pub(crate) fn ler_acao_de_abrir_pastas(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDaArvore>, ErroArvore> {
    let mut achado = None;
    for linha in html.split('\n').filter(|l| l.contains(ACAO_DE_ABRIR)) {
        let fora = ErroArvore::AcaoDeAbrirPastasForaDaFormaObservada;
        let argumentos = ler_chamada(linha, "NosAcoes[", "infraArvoreAcao").ok_or(fora)?;
        let [
            _,
            Literal::Texto(rotulo),
            Literal::Texto(alvo),
            Literal::Texto(url),
            Literal::Nulo,
            Literal::Texto(_),
            Literal::Texto(_),
            Literal::Booleano(_),
        ] = argumentos.as_slice()
        else {
            return Err(fora);
        };
        if *rotulo != format!("AP{id}") || alvo != id {
            return Err(fora);
        }
        if achado.is_some() {
            return Err(ErroArvore::AcoesDeAbrirPastasRepetidas);
        }
        achado = Some(conferir_url_de_abrir(instalacao, url, id)?);
    }
    Ok(achado)
}

/// A regra comum dos links assinados e mais: um `abrir_pastas=1`, nenhum
/// `fechar_pastas`.
fn conferir_url_de_abrir(
    instalacao: &Instalacao,
    url: &str,
    id: &str,
) -> Result<EnderecoDaArvore, ErroArvore> {
    let recusa = ErroArvore::EnderecoDeAbrirPastasNaoReconhecido;
    let absoluta = conferir_endereco_relativo(instalacao, url, "procedimento_visualizar", id)
        .map_err(|_| recusa)?;
    let consulta = url.split_once('?').map_or("", |(_, c)| c);
    let mut abrir = 0;
    for par in consulta.split('&') {
        match par.split_once('=') {
            Some(("abrir_pastas", "1")) => abrir += 1,
            Some(("abrir_pastas" | "fechar_pastas", _)) => return Err(recusa),
            _ => {}
        }
    }
    if abrir != 1 {
        return Err(recusa);
    }
    Ok(EnderecoDaArvore::conferido(absoluta))
}
