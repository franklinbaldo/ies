//! A tela "Marcadores do Processo" (`andamento_marcador_gerenciar`), lida
//! sem envio: `btnAdicionar`, `btnRemover`, `acaoRemover` e o `action` do
//! `form#frmGerenciarMarcador` nunca participam além da conferência de
//! identidade. Atributos e aninhamento pela árvore
//! `tela_html`; sem regex e sem JavaScript.
use super::{ErroMarcadores, MarcadorDoProcesso, MarcadoresLidos, TelaDosMarcadores};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::legenda::{Legenda, ler_legenda};
use crate::nivel_de_detalhe::texto_colapsado;
use std::collections::HashSet;

type Tag<'p> = crate::tela_html::Elemento<'p>;

const TITULO: &str = "SEI - Marcadores do Processo ";
const CABECALHO: [&str; 6] = [
    "",
    "Marcador",
    "Texto",
    "Usu\u{e1}rio",
    "Data/Hora",
    "A\u{e7}\u{f5}es",
];

const LINHA: ErroMarcadores = ErroMarcadores::LinhaDoMarcadorForaDaFormaObservada;

/// Lê a tela e confere a identidade (NUP e id, comparação opaca) antes de
/// qualquer linha.
pub(crate) fn ler_marcadores(
    html: &crate::tela_html::HtmlServido,
    nup: &str,
    id: &str,
) -> Result<TelaDosMarcadores, ErroMarcadores> {
    let dom = crate::tela_html::arvore(html);
    conferir_identidade(&dom, nup, id)?;
    let tabela = match dom.com_id("tblMarcadores").collect::<Vec<_>>().as_slice() {
        [] => return Ok(TelaDosMarcadores::SemTabela),
        [unica] => *unica,
        _ => return Err(ErroMarcadores::TabelasDosMarcadoresRepetidas),
    };
    if tabela.nome() != "table" || !descendentes(tabela, "table").is_empty() {
        return Err(ErroMarcadores::TabelasDosMarcadoresRepetidas);
    }
    let total_declarado = legenda(tabela)?;
    let mut ths = descendentes(tabela, "th").into_iter();
    let cabecalho_ok = ths.len() == CABECALHO.len()
        && CABECALHO.iter().enumerate().all(|(i, esperado)| {
            ths.next()
                .is_some_and(|th| rotulo_do_th(th, i == 0).as_deref() == Ok(*esperado))
        });
    if !cabecalho_ok {
        return Err(ErroMarcadores::CabecalhoDosMarcadoresNaoReconhecido);
    }
    let mut itens = Vec::new();
    let mut ids = HashSet::new();
    for tr in descendentes(tabela, "tr") {
        let celulas = filhos(tr, "td");
        if celulas.is_empty() {
            continue;
        }
        let marcador = linha(&celulas)?;
        if !ids.insert(marcador.id.clone()) {
            return Err(ErroMarcadores::MarcadorRepetidoNaTela);
        }
        itens.push(marcador);
    }
    if itens.len() > total_declarado {
        return Err(ErroMarcadores::MarcadoresAlemDoTotalDeclarado);
    }
    Ok(TelaDosMarcadores::Lidos(MarcadoresLidos {
        total_declarado,
        itens,
    }))
}

/// O único `title` começa por "SEI - Marcadores do Processo " seguido do
/// NUP; o único `form#frmGerenciarMarcador` tem `action` com uma
/// `acao=andamento_marcador_gerenciar` e o `id_procedimento` aberto.
fn conferir_identidade(
    dom: &crate::tela_html::Documento,
    nup: &str,
    id: &str,
) -> Result<(), ErroMarcadores> {
    let titulos: Vec<_> = dom.elementos("title").collect();
    let [titulo] = titulos.as_slice() else {
        return Err(ErroMarcadores::TelaDosMarcadoresAusente);
    };
    let titulo = texto(*titulo);
    let numero = titulo
        .strip_prefix(TITULO)
        .ok_or(ErroMarcadores::TelaDosMarcadoresAusente)?;
    if numero != nup {
        return Err(ErroMarcadores::NumeroDosMarcadoresDiferenteDoSolicitado);
    }
    let formularios = dom.com_id("frmGerenciarMarcador").collect::<Vec<_>>();
    let [formulario] = formularios.as_slice() else {
        return Err(ErroMarcadores::TelaDosMarcadoresAusente);
    };
    let formulario = *formulario;
    if formulario.nome() != "form" {
        return Err(ErroMarcadores::TelaDosMarcadoresAusente);
    }
    let action = formulario
        .atributo("action")
        .map(str::to_owned)
        .ok_or(ErroMarcadores::TelaDosMarcadoresAusente)?;
    let consulta = action
        .strip_prefix("controlador.php?")
        .ok_or(ErroMarcadores::TelaDosMarcadoresAusente)?;
    let pares = ConsultaDoControlador::ler(consulta)
        .map_err(ErroMarcadores::ConsultaDoFormularioIlegivel)?;
    let acao = pares
        .acao()
        .map_err(ErroMarcadores::AcaoDoFormularioNaoLida)?;
    if acao != AcaoDoSei::AndamentoMarcadorGerenciar {
        return Err(ErroMarcadores::TelaDosMarcadoresAusente);
    }
    let id_observado = pares
        .unico("id_procedimento")
        .map_err(ErroMarcadores::IdDoFormularioNaoLido)?;
    if id_observado != id {
        return Err(ErroMarcadores::IdDosMarcadoresDiferenteDoAberto);
    }
    Ok(())
}

/// `Marcadores (N registro):` ou `Marcadores (N registros):`, sem faixa.
fn legenda(tabela: Tag<'_>) -> Result<usize, ErroMarcadores> {
    let fora = || ErroMarcadores::LegendaDosMarcadoresForaDaFormaObservada;
    let [caption] = descendentes(tabela, "caption")[..] else {
        return Err(fora());
    };
    let legenda = texto(caption);
    if !legenda.starts_with("Marcadores (") {
        return Err(fora());
    }
    let Legenda { total, faixa } = ler_legenda(&legenda)?;
    if faixa.is_some() {
        return Err(fora());
    }
    Ok(total)
}

/// Rótulo de `div.infraDivRotuloOrdenacao` ou do `th`; a seleção é o `th`
/// sem texto com `#lnkInfraCheck`.
fn rotulo_do_th(th: Tag<'_>, selecao: bool) -> Result<String, ErroMarcadores> {
    let fora = || ErroMarcadores::CabecalhoDosMarcadoresNaoReconhecido;
    if selecao {
        let marcado = th.com_id("lnkInfraCheck").collect::<Vec<_>>();
        if marcado.len() != 1 {
            return Err(fora());
        }
    }
    let rotulos = nos_por_elemento_e_classe_em(th, "div", "infraDivRotuloOrdenacao");
    match rotulos.as_slice() {
        [] => Ok(texto(th)),
        [div] => Ok(texto(*div)),
        _ => Err(fora()),
    }
}

fn nos_por_elemento_e_classe_em<'p>(raiz: Tag<'p>, elemento: &str, classe: &str) -> Vec<Tag<'p>> {
    let mut achados = Vec::new();
    for tag in descendentes(raiz, elemento) {
        if tag.tem_classe(classe) {
            achados.push(tag);
        }
    }
    achados
}

/// Uma linha: seleção, Marcador, Texto, Usuário, Data/Hora, Ações.
fn linha(celulas: &[Tag<'_>]) -> Result<MarcadorDoProcesso, ErroMarcadores> {
    let [selecao, marcador, texto_td, usuario, data_hora, _acoes] = celulas[..] else {
        return Err(LINHA);
    };
    // Seleção: um `input.infraCheckbox` com `value` decimal.
    let caixas = nos_por_elemento_e_classe_em(selecao, "input", "infraCheckbox");
    let [caixa] = caixas.as_slice() else {
        return Err(LINHA);
    };
    let id = caixa.atributo("value").map(str::to_owned).ok_or(LINHA)?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(LINHA);
    }
    for ancora in descendentes(selecao, "a") {
        if let Some(id_ancora) = ancora.atributo("id").map(str::to_owned)
            && id_ancora
                .strip_prefix("lnkInfraID-")
                .is_some_and(|resto| resto != id)
        {
            return Err(LINHA);
        }
    }
    if texto_td
        .atributo("id")
        .map(str::to_owned)
        .is_some_and(|v| v != format!("tdAndamentoMarcador{id}"))
    {
        return Err(LINHA);
    }
    // Marcador: o texto da célula; o ícone dá a cor; os `title`, quando
    // presentes, repetem o nome.
    let nome = texto(marcador);
    if nome.is_empty() {
        return Err(LINHA);
    }
    let [icone] = descendentes(marcador, "img")[..] else {
        return Err(LINHA);
    };
    let cor = cor(&icone.atributo("src").map(str::to_owned).ok_or(LINHA)?)?;
    for tag in [icone, *caixa] {
        if let Some(titulo) = tag.atributo("title").map(str::to_owned)
            && texto_colapsado(&titulo) != nome
        {
            return Err(LINHA);
        }
    }
    let texto_lido = texto(texto_td);
    let data_hora = texto(data_hora);
    if !eh_data_hora(&data_hora) {
        return Err(LINHA);
    }
    Ok(MarcadorDoProcesso {
        id,
        nome,
        cor,
        texto: (!texto_lido.is_empty()).then_some(texto_lido),
        usuario: sigla(usuario)?,
        data_hora,
    })
}

/// Célula vazia: `None`; senão, exatamente um `a.ancoraSigla` com todo o texto.
fn sigla(celula: Tag<'_>) -> Result<Option<String>, ErroMarcadores> {
    let completo = texto(celula);
    let ancoras = nos_por_elemento_e_classe_em(celula, "a", "ancoraSigla");
    match ancoras.as_slice() {
        [] if completo.is_empty() => Ok(None),
        [ancora] if !completo.is_empty() && texto(*ancora) == completo => Ok(Some(completo)),
        _ => Err(LINHA),
    }
}

/// `<cor>` de `svg/marcador_<cor>.svg` (com `?…` opcional).
fn cor(src: &str) -> Result<String, ErroMarcadores> {
    let resto = src.strip_prefix("svg/marcador_").ok_or(LINHA)?;
    let (cor, depois) = resto.split_once(".svg").ok_or(LINHA)?;
    if cor.is_empty()
        || !cor.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        || !(depois.is_empty() || depois.starts_with('?'))
    {
        return Err(LINHA);
    }
    Ok(cor.to_owned())
}

/// `dd/mm/aaaa hh:mm`, só a forma (o valor é opaco).
fn eh_data_hora(texto: &str) -> bool {
    let b = texto.as_bytes();
    b.len() == 16
        && b.iter().enumerate().all(|(i, c)| match i {
            2 | 5 => *c == b'/',
            10 => *c == b' ',
            13 => *c == b':',
            _ => c.is_ascii_digit(),
        })
}

/// Entidades decodificadas (`&nbsp;` como espaço inseparável).
/// Texto visível, entidades decodificadas e espaços colapsados.
fn texto(tag: Tag<'_>) -> String {
    texto_colapsado(&tag.texto())
}

/// Descendentes `elemento` de `raiz`, na ordem do documento.
fn descendentes<'p>(raiz: Tag<'p>, elemento: &str) -> Vec<Tag<'p>> {
    raiz.elementos(elemento).collect()
}

/// Filhos diretos `elemento` de `pai`.
fn filhos<'p>(pai: Tag<'p>, elemento: &str) -> Vec<Tag<'p>> {
    pai.filhos().filter(|t| t.nome() == elemento).collect()
}
