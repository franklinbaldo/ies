//! Seleção, cadastro e reconciliação offline (#246); prévia Web no módulo fluxo.
mod fluxo;
pub(crate) use fluxo::{ErroPreviaCriacao, PedidoCriacao, PreviaCriacao, executar_criacao};

use crate::arvore_processo::DocumentoNaArvore;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::instalacao::Instalacao;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroCriar {
    ListaForaDaForma,
    TipoAusente,
    TipoAmbiguo,
    NaoAplicado,
    EfeitoIncerto,
    LinkDoTipoNaoNavegavel,
    EnderecoDoTipoNaoReconhecido,
    CadastroForaDaForma,
    TipoDoCadastroDiferente,
    DescricaoNaoOferecida,
    NivelNaoOferecido,
}
impl ErroCriar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::ListaForaDaForma => "lista_de_tipos_fora_da_forma_observada",
            Self::TipoAusente => "tipo_de_documento_nao_oferecido",
            Self::TipoAmbiguo => "tipo_de_documento_ambiguo",
            Self::NaoAplicado => "documento_criado_nao_comprovado",
            Self::EfeitoIncerto => "efeito_incerto_na_criacao_de_documento",
            Self::LinkDoTipoNaoNavegavel => "link_do_tipo_de_documento_nao_navegavel",
            Self::EnderecoDoTipoNaoReconhecido => "endereco_do_tipo_de_documento_nao_reconhecido",
            Self::CadastroForaDaForma => "cadastro_de_documento_fora_da_forma_observada",
            Self::TipoDoCadastroDiferente => "tipo_do_cadastro_diferente_do_solicitado",
            Self::DescricaoNaoOferecida => "descricao_nao_oferecida_no_cadastro",
            Self::NivelNaoOferecido => "nivel_nao_oferecido_no_cadastro",
        }
    }
}
/// Apenas nomes publicados em table#tblSeries; nunca interpreta JS como código.
pub(crate) fn tipos_oferecidos(html: &str) -> Result<Vec<String>, ErroCriar> {
    let err = ErroCriar::ListaForaDaForma;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| err)?;
    let tabelas = nos_com_id(&dom, "tblSeries").map_err(|_| err)?;
    let [tabela] = tabelas.as_slice() else {
        return Err(err);
    };
    let parser = dom.parser();
    let tabela = tabela.get(parser).and_then(tl::Node::as_tag).ok_or(err)?;
    if !tabela.name().as_utf8_str().eq_ignore_ascii_case("table") {
        return Err(err);
    }
    let mut tipos = Vec::new();
    for tag in tabela
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
            continue;
        }
        let href = valor_do_atributo(tag, "href").map_err(|_| err)?;
        if href.is_none() {
            continue;
        }
        let onclick = valor_do_atributo(tag, "onclick").map_err(|_| err)?;
        // Na 4.0 "Externo" usa documento_receber, e não documento_gerar.
        // Nas 4.1/5.0, escolher(-1) também é externo.
        let href = decodificar_atributo(href.as_deref().unwrap_or_default()).ok_or(err)?;
        let nativo =
            href.split('?').nth(1).is_some_and(|consulta| {
                consulta.split('&').any(|par| par == "acao=documento_gerar")
            }) || (href == "javascript:void(0);"
                && onclick.as_deref().is_some_and(|chamada| {
                    let chamada = chamada.trim().trim_end_matches(';');
                    chamada
                        .strip_prefix("escolher(")
                        .and_then(|v| v.strip_suffix(')'))
                        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
                }));
        if !nativo {
            continue;
        }
        let nome = tag
            .inner_text(parser)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let nome = decodificar_atributo(&nome).ok_or(err)?;
        if !nome.is_empty() {
            tipos.push(nome);
        }
    }
    Ok(tipos)
}
pub(crate) fn conferir_tipo(tipo: &str, oferecidos: &[String]) -> Result<(), ErroCriar> {
    match oferecidos.iter().filter(|s| s.as_str() == tipo).count() {
        0 => Err(ErroCriar::TipoAusente),
        1 => Ok(()),
        _ => Err(ErroCriar::TipoAmbiguo),
    }
}
/// Sucesso apenas com um id novo, número SEI e rótulo esperados, mantendo
/// exatamente todos os documentos anteriores. Falha pós-POST não é retry.
pub(crate) fn conferir_criacao(
    antes: &[DocumentoNaArvore],
    depois: &[DocumentoNaArvore],
    tipo: &str,
) -> Result<String, ErroCriar> {
    use ErroCriar as E;
    // Uma árvore inicial ambígua também inviabiliza provar unicidade:
    // não basta a leitura posterior ter IDs distintos.
    let mut originais = std::collections::HashSet::new();
    for doc in antes {
        if doc.id.is_empty() || !originais.insert(doc.id.as_str()) {
            return Err(E::EfeitoIncerto);
        }
    }
    let mut ids = std::collections::HashSet::new();
    for doc in depois {
        if doc.id.is_empty() || !ids.insert(doc.id.as_str()) {
            return Err(E::EfeitoIncerto);
        }
    }
    for original in antes {
        if !depois.iter().any(|doc| doc == original) {
            return Err(E::EfeitoIncerto);
        }
    }
    let novos: Vec<_> = depois
        .iter()
        .filter(|doc| !antes.iter().any(|d| d.id == doc.id))
        .collect();
    let novo = match novos.as_slice() {
        [] => return Err(E::NaoAplicado),
        [doc] => *doc,
        _ => return Err(E::EfeitoIncerto),
    };
    let numero = novo
        .numero_sei
        .as_deref()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(E::EfeitoIncerto)?;
    if novo.rotulo != format!("{tipo} {numero}")
        || antes
            .iter()
            .any(|d| d.numero_sei.as_deref() == Some(numero))
        || depois
            .iter()
            .filter(|d| d.numero_sei.as_deref() == Some(numero))
            .count()
            != 1
    {
        return Err(E::EfeitoIncerto);
    }
    Ok(numero.to_owned())
}


/// Aceita só o href direto e assinado publicado para o tipo exato.
/// Na forma escolher(id), a navegação depende da tela e ainda não foi derivada:
/// falha fechado, sem construir uma URL e sem executar JavaScript.
pub(crate) fn link_direto_do_tipo(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    tipo: &str,
) -> Result<String, ErroCriar> {
    use ErroCriar as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::ListaForaDaForma)?;
    let tabelas = nos_com_id(&dom, "tblSeries").map_err(|_| E::ListaForaDaForma)?;
    let [tabela] = tabelas.as_slice() else {
        return Err(E::ListaForaDaForma);
    };
    let tabela = tabela.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::ListaForaDaForma)?;
    if !tabela.name().as_utf8_str().eq_ignore_ascii_case("table") {
        return Err(E::ListaForaDaForma);
    }
    let mut links = Vec::new();
    for tag in tabela.children().all(dom.parser()).iter().filter_map(tl::Node::as_tag) {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
            continue;
        }
        let nome = tag.inner_text(dom.parser()).split_whitespace().collect::<Vec<_>>().join(" ");
        let nome = decodificar_atributo(&nome).ok_or(E::ListaForaDaForma)?;
        if nome != tipo {
            continue;
        }
        let href = valor_do_atributo(tag, "href").map_err(|_| E::ListaForaDaForma)?
            .ok_or(E::LinkDoTipoNaoNavegavel)?;
        let href = decodificar_atributo(&href).ok_or(E::ListaForaDaForma)?;
        links.push(href);
    }
    let link = match links.as_slice() {
        [] => return Err(E::TipoAusente),
        [link] => link,
        _ => return Err(E::TipoAmbiguo),
    };
    if link.starts_with("javascript:") {
        return Err(E::LinkDoTipoNaoNavegavel);
    }
    conferir_endereco_relativo(instalacao, link, "documento_gerar", id_procedimento)
        .map_err(|_| E::EnderecoDoTipoNaoReconhecido)
}

/// Verifica a identidade do cadastro antes de qualquer POST de criação.
pub(crate) fn conferir_cadastro(
    html: &str,
    tipo: &str,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    use ErroCriar as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::CadastroForaDaForma)?;
    let formularios = nos_com_id(&dom, "frmDocumentoCadastro").map_err(|_| E::CadastroForaDaForma)?;
    let [formulario] = formularios.as_slice() else {
        return Err(E::CadastroForaDaForma);
    };
    let formulario = formulario.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::CadastroForaDaForma)?;
    if !formulario.name().as_utf8_str().eq_ignore_ascii_case("form")
        || valor_do_atributo(formulario, "method").map_err(|_| E::CadastroForaDaForma)?
            .as_deref() != Some("post") {
        return Err(E::CadastroForaDaForma);
    }
    let rotulos = nos_com_id(&dom, "lblSerieTitulo").map_err(|_| E::CadastroForaDaForma)?;
    let [rotulo] = rotulos.as_slice() else {
        return Err(E::CadastroForaDaForma);
    };
    let rotulo = rotulo.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::CadastroForaDaForma)?;
    if !rotulo.name().as_utf8_str().eq_ignore_ascii_case("label") {
        return Err(E::CadastroForaDaForma);
    }
    let lido = rotulo.inner_text(dom.parser()).split_whitespace().collect::<Vec<_>>().join(" ");
    let lido = decodificar_atributo(&lido).ok_or(E::CadastroForaDaForma)?;
    if lido != tipo {
        return Err(E::TipoDoCadastroDiferente);
    }
    if descricao.is_some_and(|valor| !valor.is_empty()) {
        let controles = nos_com_id(&dom, "txtDescricao").map_err(|_| E::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(E::DescricaoNaoOferecida);
        };
        let controle = controle.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::DescricaoNaoOferecida)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || valor_do_atributo(controle, "name").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some("txtDescricao")
            || valor_do_atributo(controle, "disabled").map_err(|_| E::CadastroForaDaForma)?.is_some()
        {
            return Err(E::DescricaoNaoOferecida);
        }
    }
    if let Some(nivel) = nivel {
        let (id, valor) = match nivel {
            "publico" => ("optPublico", "0"),
            "restrito" => ("optRestrito", "1"),
            "sigiloso" => ("optSigiloso", "2"),
            _ => return Err(E::NivelNaoOferecido),
        };
        let controles = nos_com_id(&dom, id).map_err(|_| E::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(E::NivelNaoOferecido);
        };
        let controle = controle.get(dom.parser()).and_then(tl::Node::as_tag).ok_or(E::NivelNaoOferecido)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || valor_do_atributo(controle, "name").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some("rdoNivelAcesso")
            || valor_do_atributo(controle, "type").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some("radio")
            || valor_do_atributo(controle, "value").map_err(|_| E::CadastroForaDaForma)?
                .as_deref() != Some(valor)
            || valor_do_atributo(controle, "disabled").map_err(|_| E::CadastroForaDaForma)?.is_some()
        {
            return Err(E::NivelNaoOferecido);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
