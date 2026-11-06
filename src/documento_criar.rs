//! Seleção, cadastro e reconciliação offline (#246); prévia Web no módulo fluxo.
#[cfg(test)]
mod acervo;
mod confirmacao;
mod fluxo;
mod guarda_script;
mod link_do_tipo;
pub(crate) use fluxo::{ErroPreviaCriacao, PedidoCriacao, PreviaCriacao, executar_criacao};

use crate::arvore_processo::DocumentoNaArvore;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroCriar {
    #[causa(nome = "lista_de_tipos_fora_da_forma_observada")]
    ListaForaDaForma,
    #[causa(nome = "tipo_de_documento_nao_oferecido")]
    TipoAusente,
    #[causa(nome = "tipo_de_documento_ambiguo")]
    TipoAmbiguo,
    #[causa(nome = "documento_criado_nao_comprovado")]
    NaoAplicado,
    #[causa(
        nome = "efeito_incerto_na_criacao_de_documento",
        acao = "conferir_escrita"
    )]
    EfeitoIncerto,
    #[causa(nome = "link_do_tipo_de_documento_nao_navegavel")]
    LinkDoTipoNaoNavegavel,
    #[causa(nome = "endereco_do_tipo_de_documento_nao_reconhecido")]
    EnderecoDoTipoNaoReconhecido,
    #[causa(nome = "cadastro_de_documento_fora_da_forma_observada")]
    CadastroForaDaForma,
    #[causa(nome = "tipo_do_cadastro_diferente_do_solicitado")]
    TipoDoCadastroDiferente,
    #[causa(nome = "descricao_nao_oferecida_no_cadastro")]
    DescricaoNaoOferecida,
    #[causa(nome = "nivel_nao_oferecido_no_cadastro")]
    NivelNaoOferecido,
    #[causa(nome = "campo_obrigatorio_do_cadastro_nao_atendido")]
    CampoObrigatorioNaoAtendido,
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
            }) || (href == "#" || href == "javascript:void(0);")
                && onclick.as_deref().and_then(id_da_escolha_literal).is_some();
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

/// A 4.0 publica URL direta; a 4.1/5.0 publica escolher(id).
/// Nunca carrega credenciais de sessão nem URL assinada no Debug.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum EscolhaDoDocumento {
    Link(String),
    Formulario(String),
}

/// Não expõe URL assinada ou identificadores sensíveis no Debug.
impl std::fmt::Debug for EscolhaDoDocumento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Link(_) => f.write_str("Link(<redigido>)"),
            Self::Formulario(_) => f.write_str("Formulario(<redigido>)"),
        }
    }
}

fn id_da_escolha_literal(onclick: &str) -> Option<String> {
    let chamada = onclick.trim().trim_end_matches(';').trim();
    let id = chamada.strip_prefix("escolher(")?.strip_suffix(')')?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.to_owned())
}

/// Reconhece só o corpo literal observado da função, com duas instruções
/// (reescrita do id e submit da lista). Texto de JS não é executado.
fn script_prova_escolha(html: &str) -> bool {
    let compacto: String = html.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    // Não aceitar outra definição com o mesmo nome: poderia sobrescrever a
    // função observada no navegador, e nosso POST já não seria equivalente.
    compacto.matches("functionescolher(").count() == 1
        && compacto.contains(
            "functionescolher(id){document.getElementById('hdnIdSerie').value=id;document.getElementById('frmDocumentoEscolherTipo').submit();}",
        )
}

/// A lista inicial pode mostrar apenas tipos recentes. O frontend 4.0/4.1
/// declara o botão "Exibir todos" por exibirSeries('T'), que altera apenas
/// o filtro da sessão. Nunca executar JS ou inferir a função pelo botão.
pub(crate) fn filtro_de_todos(html: &str) -> Result<bool, ErroCriar> {
    use ErroCriar as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::ListaForaDaForma)?;
    let botoes = nos_com_id(&dom, "ancExibirSeries").map_err(|_| E::ListaForaDaForma)?;
    let [botao] = botoes.as_slice() else {
        return if botoes.is_empty() {
            Ok(false)
        } else {
            Err(E::ListaForaDaForma)
        };
    };
    let tag = botao
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(E::ListaForaDaForma)?;
    if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
        return Err(E::ListaForaDaForma);
    }
    let href = valor_do_atributo(tag, "href")
        .map_err(|_| E::ListaForaDaForma)?
        .ok_or(E::LinkDoTipoNaoNavegavel)?;
    let href = decodificar_atributo(&href).ok_or(E::ListaForaDaForma)?;
    let onclick = valor_do_atributo(tag, "onclick")
        .map_err(|_| E::ListaForaDaForma)?
        .ok_or(E::LinkDoTipoNaoNavegavel)?;
    let onclick = decodificar_atributo(&onclick).ok_or(E::ListaForaDaForma)?;
    let chamada = onclick.trim().trim_end_matches(';').trim();
    if (href != "#" && href != "javascript:void(0);")
        || (chamada != "exibirSeries('T')" && chamada != "exibirSeries(\"T\")")
    {
        return Err(E::LinkDoTipoNaoNavegavel);
    }
    let compacto: String = html.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if compacto.matches("functionexibirSeries(").count() != 1
        || !compacto.contains(
            "functionexibirSeries(tipo){document.getElementById('hdnFiltroSerie').value=tipo;document.getElementById('frmDocumentoEscolherTipo').submit();}",
        )
    {
        return Err(E::LinkDoTipoNaoNavegavel);
    }
    Ok(true)
}

/// Escolhe o tipo exato na lista observada, sem inventar URL ou executar JS.
/// O tipo externo (escolher(-1)) não é documento interno.
pub(crate) fn escolha_do_tipo(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    tipo: &str,
) -> Result<EscolhaDoDocumento, ErroCriar> {
    use ErroCriar as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::ListaForaDaForma)?;
    let tabelas = nos_com_id(&dom, "tblSeries").map_err(|_| E::ListaForaDaForma)?;
    let [tabela] = tabelas.as_slice() else {
        return Err(E::ListaForaDaForma);
    };
    let tabela = tabela
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(E::ListaForaDaForma)?;
    if !tabela.name().as_utf8_str().eq_ignore_ascii_case("table") {
        return Err(E::ListaForaDaForma);
    }
    let mut links = Vec::new();
    for tag in tabela
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("a") {
            continue;
        }
        let nome = tag
            .inner_text(dom.parser())
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let nome = decodificar_atributo(&nome).ok_or(E::ListaForaDaForma)?;
        if nome != tipo {
            continue;
        }
        let onclick = valor_do_atributo(tag, "onclick").map_err(|_| E::ListaForaDaForma)?;
        // A variante -1 é documento externo, não um tipo nativo.
        if onclick
            .as_deref()
            .is_some_and(|s| s.trim().trim_end_matches(';') == "escolher(-1)")
        {
            continue;
        }
        let href = valor_do_atributo(tag, "href")
            .map_err(|_| E::ListaForaDaForma)?
            .ok_or(E::LinkDoTipoNaoNavegavel)?;
        let href = decodificar_atributo(&href).ok_or(E::ListaForaDaForma)?;
        links.push((href, onclick));
    }
    let (href, onclick) = match links.as_slice() {
        [] => return Err(E::TipoAusente),
        [link] => link,
        _ => return Err(E::TipoAmbiguo),
    };
    if href.starts_with("controlador.php?") {
        let link = link_do_tipo::conferir_link_do_tipo(instalacao, href, id_procedimento)?;
        return Ok(EscolhaDoDocumento::Link(link));
    }
    if href != "#" && href != "javascript:void(0);" {
        return Err(E::LinkDoTipoNaoNavegavel);
    }
    let id = onclick
        .as_deref()
        .and_then(id_da_escolha_literal)
        .ok_or(E::LinkDoTipoNaoNavegavel)?;
    if !script_prova_escolha(html) {
        return Err(E::LinkDoTipoNaoNavegavel);
    }
    Ok(EscolhaDoDocumento::Formulario(id))
}

/// O redirecionamento do POST de escolha só pode abrir o cadastro assinado
/// do mesmo processo. Suporta Location relativa ou absoluta da instalação.
pub(crate) fn cadastro_redirecionado(
    instalacao: &Instalacao,
    location: &str,
    id_procedimento: &str,
) -> Result<String, ErroCriar> {
    let base = format!("{}/", instalacao.url_base());
    let relativo = location.strip_prefix(&base).unwrap_or(location);
    conferir_endereco_relativo(instalacao, relativo, "documento_gerar", id_procedimento)
        .map_err(|_| ErroCriar::EnderecoDoTipoNaoReconhecido)
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
    let formularios =
        nos_com_id(&dom, "frmDocumentoCadastro").map_err(|_| E::CadastroForaDaForma)?;
    let [formulario] = formularios.as_slice() else {
        return Err(E::CadastroForaDaForma);
    };
    let formulario = formulario
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(E::CadastroForaDaForma)?;
    if !formulario.name().as_utf8_str().eq_ignore_ascii_case("form")
        || valor_do_atributo(formulario, "method")
            .map_err(|_| E::CadastroForaDaForma)?
            .as_deref()
            != Some("post")
    {
        return Err(E::CadastroForaDaForma);
    }
    let rotulos = nos_com_id(&dom, "lblSerieTitulo").map_err(|_| E::CadastroForaDaForma)?;
    let [rotulo] = rotulos.as_slice() else {
        return Err(E::CadastroForaDaForma);
    };
    let rotulo = rotulo
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(E::CadastroForaDaForma)?;
    if !rotulo.name().as_utf8_str().eq_ignore_ascii_case("label") {
        return Err(E::CadastroForaDaForma);
    }
    let lido = rotulo
        .inner_text(dom.parser())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let lido = decodificar_atributo(&lido).ok_or(E::CadastroForaDaForma)?;
    if lido != tipo {
        return Err(E::TipoDoCadastroDiferente);
    }
    if descricao.is_some_and(|valor| !valor.is_empty()) {
        let controles = nos_com_id(&dom, "txtDescricao").map_err(|_| E::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(E::DescricaoNaoOferecida);
        };
        let controle = controle
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(E::DescricaoNaoOferecida)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || valor_do_atributo(controle, "name")
                .map_err(|_| E::CadastroForaDaForma)?
                .as_deref()
                != Some("txtDescricao")
            || crate::html_atributos::atributo_presente(controle, "disabled")
                .map_err(|_| E::CadastroForaDaForma)?
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
        let controle = controle
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(E::NivelNaoOferecido)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || valor_do_atributo(controle, "name")
                .map_err(|_| E::CadastroForaDaForma)?
                .as_deref()
                != Some("rdoNivelAcesso")
            || valor_do_atributo(controle, "type")
                .map_err(|_| E::CadastroForaDaForma)?
                .as_deref()
                != Some("radio")
            || valor_do_atributo(controle, "value")
                .map_err(|_| E::CadastroForaDaForma)?
                .as_deref()
                != Some(valor)
            || crate::html_atributos::atributo_presente(controle, "disabled")
                .map_err(|_| E::CadastroForaDaForma)?
        {
            return Err(E::NivelNaoOferecido);
        }
    }
    if nivel.is_some() {
        conferir_controles_obrigatorios(&dom, formulario, descricao, nivel)?;
    }
    Ok(())
}

/// Recusa controles obrigatórios que não possuem valor da operação.
/// `required` e `disabled` são booleanos: sua presença é verificada
/// por `html_atributos`, inclusive sem sinal de igualdade.
fn conferir_controles_obrigatorios(
    dom: &tl::VDom<'_>,
    formulario: &tl::HTMLTag<'_>,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    use ErroCriar as E;
    for tag in formulario
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let nome_tag = tag.name().as_utf8_str();
        if !nome_tag.eq_ignore_ascii_case("input")
            && !nome_tag.eq_ignore_ascii_case("select")
            && !nome_tag.eq_ignore_ascii_case("textarea")
        {
            continue;
        }
        if !crate::html_atributos::atributo_presente(tag, "required")
            .map_err(|_| E::CadastroForaDaForma)?
            || crate::html_atributos::atributo_presente(tag, "disabled")
                .map_err(|_| E::CadastroForaDaForma)?
        {
            continue;
        }
        let tipo_controle = valor_do_atributo(tag, "type").map_err(|_| E::CadastroForaDaForma)?;
        if matches!(
            tipo_controle.as_deref(),
            Some("hidden" | "submit" | "button" | "reset" | "image")
        ) {
            continue;
        }
        let nome = valor_do_atributo(tag, "name").map_err(|_| E::CadastroForaDaForma)?;
        let atendido = match nome.as_deref() {
            Some("txtDescricao") => descricao.is_some_and(|s| !s.trim().is_empty()),
            Some("rdoNivelAcesso") => nivel.is_some(),
            _ => false,
        };
        if !atendido {
            return Err(E::CampoObrigatorioNaoAtendido);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_scripts_inertes;
