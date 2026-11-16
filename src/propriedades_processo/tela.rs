//! O formulário `form#frmProcedimentoCadastro` servido por
//! `procedimento_alterar` (ou `procedimento_consultar`), lido sem envio:
//! nenhum campo é reescrito e `btnSalvar` nunca participa. Atributos pelo
//! árvore `tela_html` (atributos e aninhamento como no navegador).
use super::{Assunto, ErroPropriedades, FonteDasPropriedades, NivelDeAcesso, PropriedadesLidas};

const FORMULARIO: &str = "frmProcedimentoCadastro";

type Tag<'p> = crate::tela_html::Elemento<'p>;

/// Lê a tela e confere a identidade (NUP e id, comparação opaca) antes de
/// qualquer outro campo.
pub(crate) fn ler_propriedades(
    html: &crate::tela_html::HtmlServido,
    nup: &str,
    id: &str,
    fonte: FonteDasPropriedades,
) -> Result<PropriedadesLidas, ErroPropriedades> {
    let dom = crate::tela_html::arvore(html);
    let formulario = match dom.com_id(FORMULARIO).collect::<Vec<_>>().as_slice() {
        [] => return Err(ErroPropriedades::FormularioDasPropriedadesAusente),
        [unico] => *unico,
        _ => return Err(ErroPropriedades::FormulariosDasPropriedadesRepetidos),
    };
    let campos = Campos { formulario };
    if valor(campos.unico("hdnProtocoloFormatado")?) != nup {
        return Err(ErroPropriedades::NumeroDasPropriedadesDiferenteDoSolicitado);
    }
    if valor(campos.unico("hdnIdProcedimento")?) != id {
        return Err(ErroPropriedades::IdDasPropriedadesDiferenteDoAberto);
    }
    for oculta in ["hdnAssuntos", "hdnInteressadosProcedimento"] {
        if let Some(tag) = campos.opcional(oculta)?
            && !valor(tag).is_empty()
        {
            return Err(ErroPropriedades::ListaOcultaDasPropriedadesPreenchida);
        }
    }
    Ok(PropriedadesLidas {
        tipo: tipo(&campos)?,
        especificacao: valor(campos.unico("txtDescricao")?),
        interessados: interessados(&campos)?,
        assuntos: assuntos(&campos)?,
        observacoes: observacoes(&campos)?,
        nivel_de_acesso: nivel_de_acesso(&campos)?,
        hipotese_legal: marcada(campos.unico("selHipoteseLegal")?)?.map(|(_, t)| t),
        data_de_geracao: data_de_geracao(&campos)?,
        fonte,
    })
}

struct Campos<'p> {
    formulario: Tag<'p>,
}

impl<'p> Campos<'p> {
    fn opcional(&self, id: &str) -> Result<Option<Tag<'p>>, ErroPropriedades> {
        let nos = self.formulario.com_id(id).collect::<Vec<_>>();
        match nos.as_slice() {
            [] => Ok(None),
            [unico] => Ok(Some(*unico)),
            _ => Err(ErroPropriedades::CampoDasPropriedadesRepetido),
        }
    }

    fn unico(&self, id: &str) -> Result<Tag<'p>, ErroPropriedades> {
        self.opcional(id)?
            .ok_or(ErroPropriedades::CampoDasPropriedadesAusente)
    }
}

/// `(value, selected, texto)` de cada `option` de `select`, em ordem.
fn opcoes(select: Tag<'_>) -> Vec<(String, bool, String)> {
    let mut opcoes = Vec::new();
    for no in select.elementos("option") {
        let valor = no.atributo("value").unwrap_or_default().to_owned();
        let texto = espacos(&no.texto());
        opcoes.push((valor, no.presente("selected"), texto));
    }
    opcoes
}

fn espacos(texto: &str) -> String {
    texto
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `value` decodificado; sem o atributo, vazio.
fn valor(tag: Tag<'_>) -> String {
    let bruto = tag.atributo("value");
    bruto.map(str::to_owned).unwrap_or_default()
}

/// A única `option` marcada com `value` diferente de `null`; duas: recusa.
fn marcada(select: Tag<'_>) -> Result<Option<(String, String)>, ErroPropriedades> {
    let marcadas: Vec<(String, String)> = opcoes(select)
        .into_iter()
        .filter(|(_, marcada, _)| *marcada)
        .map(|(valor, _, texto)| (valor, texto))
        .collect();
    match marcadas.as_slice() {
        [] => Ok(None),
        [(valor, _)] if valor == "null" => Ok(None),
        [unica] => Ok(Some(unica.clone())),
        _ => Err(ErroPropriedades::OpcoesMarcadasRepetidas),
    }
}

fn tipo(campos: &Campos<'_>) -> Result<String, ErroPropriedades> {
    let Some((valor_marcado, texto)) = marcada(campos.unico("selTipoProcedimento")?)? else {
        return Err(ErroPropriedades::TipoDoProcessoNaoMarcado);
    };
    if let Some(oculto) = campos.opcional("hdnIdTipoProcedimento")?
        && valor(oculto) != valor_marcado
    {
        return Err(ErroPropriedades::TipoDoProcessoInconsistente);
    }
    Ok(texto)
}

fn assuntos(campos: &Campos<'_>) -> Result<Vec<Assunto>, ErroPropriedades> {
    let select = campos.unico("selAssuntos")?;
    let mut lidos = Vec::new();
    for (valor, _, texto) in opcoes(select) {
        if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) || texto.is_empty() {
            return Err(ErroPropriedades::AssuntoForaDaFormaObservada);
        }
        let codigo = texto.split_once(" - ").filter(|(codigo, descricao)| {
            !descricao.is_empty()
                && codigo.bytes().any(|b| b.is_ascii_digit())
                && codigo.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        });
        let Some((codigo, descricao)) = codigo else {
            return Err(ErroPropriedades::CodigoDoAssuntoNaoInterpretado);
        };
        lidos.push(Assunto {
            codigo: codigo.to_owned(),
            descricao: descricao.to_owned(),
        });
    }
    Ok(lidos)
}

fn interessados(campos: &Campos<'_>) -> Result<Vec<String>, ErroPropriedades> {
    let select = campos.unico("selInteressadosProcedimento")?;
    opcoes(select)
        .into_iter()
        .map(|(_, _, texto)| {
            if texto.is_empty() {
                Err(ErroPropriedades::InteressadoForaDaFormaObservada)
            } else {
                Ok(texto)
            }
        })
        .collect()
}

/// O texto do `textarea`, sem a quebra de linha que o HTML descarta logo
/// depois da tag de abertura.
fn observacoes(campos: &Campos<'_>) -> Result<String, ErroPropriedades> {
    let tag = campos.unico("txaObservacoes")?;
    let texto = tag.texto();
    let texto = texto
        .strip_prefix("\r\n")
        .or_else(|| texto.strip_prefix('\n'))
        .unwrap_or(&texto);
    Ok(texto.to_owned())
}

/// Os `rdoNivelAcesso` do formulário: um marcado, `value` e `id` observados.
fn nivel_de_acesso(campos: &Campos<'_>) -> Result<NivelDeAcesso, ErroPropriedades> {
    let mut marcados = Vec::new();
    for no in campos.formulario.elementos("input") {
        let nome = no.atributo("name");
        if nome != Some("rdoNivelAcesso") || !no.presente("checked") {
            continue;
        }
        let valor = no.atributo("value");
        let id = no.atributo("id");
        marcados.push((valor, id));
    }
    match marcados.as_slice() {
        [] => Err(ErroPropriedades::NivelDeAcessoNaoMarcado),
        [(Some("0"), Some("optPublico"))] => Ok(NivelDeAcesso::Publico),
        [(Some("1"), Some("optRestrito"))] => Ok(NivelDeAcesso::Restrito),
        [(Some("2"), Some("optSigiloso"))] => Ok(NivelDeAcesso::Sigiloso),
        _ => Err(ErroPropriedades::NivelDeAcessoForaDaFormaObservada),
    }
}

/// `hdnDtaGeracao` na forma `dd/mm/aaaa`; ausente ou vazio é recusa.
fn data_de_geracao(campos: &Campos<'_>) -> Result<String, ErroPropriedades> {
    let Some(tag) = campos.opcional("hdnDtaGeracao")? else {
        return Err(ErroPropriedades::DataDeGeracaoAusente);
    };
    let data = valor(tag);
    if data.is_empty() {
        return Err(ErroPropriedades::DataDeGeracaoAusente);
    }
    let forma = data.len() == 10
        && data.bytes().enumerate().all(|(i, b)| match i {
            2 | 5 => b == b'/',
            _ => b.is_ascii_digit(),
        });
    if !forma {
        return Err(ErroPropriedades::DataDeGeracaoForaDaFormaObservada);
    }
    Ok(data)
}
