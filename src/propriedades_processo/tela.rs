//! O formulário `form#frmProcedimentoCadastro` servido por
//! `procedimento_alterar` (ou `procedimento_consultar`), lido sem envio:
//! nenhum campo é reescrito e `btnSalvar` nunca participa. Atributos pelo
//! tokenizador de `html_atributos` (o `tl` 0.7.8 corrompe o atributo depois
//! de um booleano, como `disabled` do `optSigiloso`); aninhamento pelo `tl`.
use super::{
    Assunto, ErroPropriedades as E, FonteDasPropriedades, NivelDeAcesso, PropriedadesLidas,
};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributos, nos_com_id, nos_com_id_em};

const FORMULARIO: &str = "frmProcedimentoCadastro";

type Tag<'p> = &'p tl::HTMLTag<'p>;

/// Lê a tela e confere a identidade (NUP e id, comparação opaca) antes de
/// qualquer outro campo.
pub(crate) fn ler_propriedades(
    html: &str,
    nup: &str,
    id: &str,
    fonte: FonteDasPropriedades,
) -> Result<PropriedadesLidas, E> {
    let ilegivel = E::HtmlDasPropriedadesNaoInterpretado;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ilegivel)?;
    let parser = dom.parser();
    let formulario = match nos_com_id(&dom, FORMULARIO)
        .map_err(|_| ilegivel)?
        .as_slice()
    {
        [] => return Err(E::FormularioDasPropriedadesAusente),
        [unico] => unico
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ilegivel)?,
        _ => return Err(E::FormulariosDasPropriedadesRepetidos),
    };
    let campos = Campos { formulario, parser };
    if valor(campos.unico("hdnProtocoloFormatado")?)? != nup {
        return Err(E::NumeroDasPropriedadesDiferenteDoSolicitado);
    }
    if valor(campos.unico("hdnIdProcedimento")?)? != id {
        return Err(E::IdDasPropriedadesDiferenteDoAberto);
    }
    for oculta in ["hdnAssuntos", "hdnInteressadosProcedimento"] {
        if let Some(tag) = campos.opcional(oculta)?
            && !valor(tag)?.is_empty()
        {
            return Err(E::ListaOcultaDasPropriedadesNaoObservada);
        }
    }
    Ok(PropriedadesLidas {
        tipo: tipo(&campos)?,
        especificacao: valor(campos.unico("txtDescricao")?)?,
        interessados: interessados(&campos)?,
        assuntos: assuntos(&campos)?,
        observacoes: observacoes(&campos)?,
        nivel_de_acesso: nivel_de_acesso(&campos)?,
        hipotese_legal: marcada(&campos, campos.unico("selHipoteseLegal")?)?.map(|(_, t)| t),
        data_de_geracao: data_de_geracao(&campos)?,
        fonte,
    })
}

struct Campos<'p> {
    formulario: Tag<'p>,
    parser: &'p tl::Parser<'p>,
}

impl<'p> Campos<'p> {
    fn opcional(&self, id: &str) -> Result<Option<Tag<'p>>, E> {
        let nos = nos_com_id_em(self.formulario, self.parser, id)
            .map_err(|_| E::HtmlDasPropriedadesNaoInterpretado)?;
        match nos.as_slice() {
            [] => Ok(None),
            [unico] => unico
                .get(self.parser)
                .and_then(tl::Node::as_tag)
                .map(Some)
                .ok_or(E::HtmlDasPropriedadesNaoInterpretado),
            _ => Err(E::CampoDasPropriedadesRepetido),
        }
    }

    fn unico(&self, id: &str) -> Result<Tag<'p>, E> {
        self.opcional(id)?.ok_or(E::CampoDasPropriedadesAusente)
    }

    /// `(value, selected, texto)` de cada `option` de `select`, em ordem.
    fn opcoes(&self, select: Tag<'p>) -> Result<Vec<(String, bool, String)>, E> {
        let mut opcoes = Vec::new();
        for no in select
            .query_selector(self.parser, "option")
            .into_iter()
            .flatten()
        {
            let Some(tag) = no.get(self.parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let lidos = atributos(tag).map_err(|_| E::HtmlDasPropriedadesNaoInterpretado)?;
            let bruto = lidos
                .valor("value")
                .map_err(|_| E::HtmlDasPropriedadesNaoInterpretado)?;
            let valor = bruto.map(decodificar).transpose()?.unwrap_or_default();
            let texto = espacos(&decodificar(&tag.inner_text(self.parser))?);
            opcoes.push((valor, lidos.presente("selected"), texto));
        }
        Ok(opcoes)
    }
}

/// Entidades decodificadas (`&nbsp;` como espaço inseparável).
fn decodificar(bruto: &str) -> Result<String, E> {
    decodificar_atributo(&bruto.replace("&nbsp;", "\u{a0}"))
        .ok_or(E::TextoDasPropriedadesNaoDecodificado)
}

fn espacos(texto: &str) -> String {
    texto
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `value` decodificado; sem o atributo, vazio.
fn valor(tag: Tag<'_>) -> Result<String, E> {
    let lidos = atributos(tag).map_err(|_| E::HtmlDasPropriedadesNaoInterpretado)?;
    let bruto = lidos
        .valor("value")
        .map_err(|_| E::CampoDasPropriedadesRepetido)?;
    bruto
        .map(decodificar)
        .transpose()
        .map(Option::unwrap_or_default)
}

/// A única `option` marcada com `value` diferente de `null`; duas: recusa.
fn marcada(campos: &Campos<'_>, select: Tag<'_>) -> Result<Option<(String, String)>, E> {
    let marcadas: Vec<(String, String)> = campos
        .opcoes(select)?
        .into_iter()
        .filter(|(_, marcada, _)| *marcada)
        .map(|(valor, _, texto)| (valor, texto))
        .collect();
    match marcadas.as_slice() {
        [] => Ok(None),
        [(valor, _)] if valor == "null" => Ok(None),
        [unica] => Ok(Some(unica.clone())),
        _ => Err(E::OpcoesMarcadasRepetidas),
    }
}

fn tipo(campos: &Campos<'_>) -> Result<Option<String>, E> {
    let Some((valor_marcado, texto)) = marcada(campos, campos.unico("selTipoProcedimento")?)?
    else {
        return Ok(None);
    };
    if let Some(oculto) = campos.opcional("hdnIdTipoProcedimento")?
        && valor(oculto)? != valor_marcado
    {
        return Err(E::TipoDoProcessoInconsistente);
    }
    Ok(Some(texto))
}

fn assuntos(campos: &Campos<'_>) -> Result<Vec<Assunto>, E> {
    let select = campos.unico("selAssuntos")?;
    let mut lidos = Vec::new();
    for (valor, _, texto) in campos.opcoes(select)? {
        if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) || texto.is_empty() {
            return Err(E::AssuntoForaDaFormaObservada);
        }
        let codigo = texto.split_once(" - ").filter(|(codigo, descricao)| {
            !descricao.is_empty()
                && codigo.bytes().any(|b| b.is_ascii_digit())
                && codigo.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        });
        lidos.push(match codigo {
            Some((codigo, descricao)) => Assunto {
                codigo: Some(codigo.to_owned()),
                descricao: descricao.to_owned(),
            },
            None => Assunto {
                codigo: None,
                descricao: texto,
            },
        });
    }
    Ok(lidos)
}

fn interessados(campos: &Campos<'_>) -> Result<Vec<String>, E> {
    let select = campos.unico("selInteressadosProcedimento")?;
    campos
        .opcoes(select)?
        .into_iter()
        .map(|(_, _, texto)| {
            if texto.is_empty() {
                Err(E::InteressadoForaDaFormaObservada)
            } else {
                Ok(texto)
            }
        })
        .collect()
}

/// O texto do `textarea`, sem a quebra de linha que o HTML descarta logo
/// depois da tag de abertura.
fn observacoes(campos: &Campos<'_>) -> Result<String, E> {
    let tag = campos.unico("txaObservacoes")?;
    let texto = decodificar(&tag.inner_text(campos.parser))?;
    let texto = texto
        .strip_prefix("\r\n")
        .or_else(|| texto.strip_prefix('\n'))
        .unwrap_or(&texto);
    Ok(texto.to_owned())
}

/// Os `rdoNivelAcesso` do formulário: um marcado, `value` e `id` observados.
fn nivel_de_acesso(campos: &Campos<'_>) -> Result<NivelDeAcesso, E> {
    let mut marcados = Vec::new();
    let entradas = campos.formulario.query_selector(campos.parser, "input");
    for no in entradas.into_iter().flatten() {
        let Some(tag) = no.get(campos.parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(|_| E::HtmlDasPropriedadesNaoInterpretado)?;
        let nome = lidos
            .valor("name")
            .map_err(|_| E::HtmlDasPropriedadesNaoInterpretado)?;
        if nome != Some("rdoNivelAcesso") || !lidos.presente("checked") {
            continue;
        }
        let valor = lidos.valor("value").ok().flatten();
        let id = lidos.valor("id").ok().flatten();
        marcados.push((valor, id));
    }
    match marcados.as_slice() {
        [] => Err(E::NivelDeAcessoNaoMarcado),
        [(Some("0"), Some("optPublico"))] => Ok(NivelDeAcesso::Publico),
        [(Some("1"), Some("optRestrito"))] => Ok(NivelDeAcesso::Restrito),
        [(Some("2"), Some("optSigiloso"))] => Ok(NivelDeAcesso::Sigiloso),
        _ => Err(E::NivelDeAcessoForaDaFormaObservada),
    }
}

/// `hdnDtaGeracao` na forma `dd/mm/aaaa`; ausente ou vazio, `None`.
fn data_de_geracao(campos: &Campos<'_>) -> Result<Option<String>, E> {
    let Some(tag) = campos.opcional("hdnDtaGeracao")? else {
        return Ok(None);
    };
    let data = valor(tag)?;
    if data.is_empty() {
        return Ok(None);
    }
    let forma = data.len() == 10
        && data.bytes().enumerate().all(|(i, b)| match i {
            2 | 5 => b == b'/',
            _ => b.is_ascii_digit(),
        });
    if !forma {
        return Err(E::DataDeGeracaoForaDaFormaObservada);
    }
    Ok(Some(data))
}
