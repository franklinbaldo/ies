//! O formulário `form#frmProcedimentoCadastro` servido por
//! `procedimento_alterar` (ou `procedimento_consultar`), lido sem envio:
//! nenhum campo é reescrito e `btnSalvar` nunca participa. Atributos pelo
//! árvore `tela_html` (atributos e aninhamento como no navegador).
use super::{
    Assunto, ErroPropriedades, FonteDasPropriedades, Interessado, NivelDeAcesso, PropriedadesLidas,
};

use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, valor_da_lupa,
};

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
    let (id_tipo, tipo) = tipo(&campos)?;
    let especificacao = valor(campos.unico("txtDescricao")?);
    let interessados = interessados(&campos)?;
    let assuntos = assuntos(&campos)?;
    let observacoes = observacoes(&campos)?;
    let nivel_de_acesso = nivel_de_acesso(&campos)?;
    let (id_hipotese_legal, hipotese_legal) = marcada(campos.unico("selHipoteseLegal")?)?
        .map_or((None, None), |(id, texto)| (Some(id), Some(texto)));
    Ok(PropriedadesLidas {
        id_tipo,
        tipo,
        especificacao,
        interessados,
        assuntos,
        observacoes,
        nivel_de_acesso,
        id_hipotese_legal,
        hipotese_legal,
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

fn tipo(campos: &Campos<'_>) -> Result<(String, String), ErroPropriedades> {
    let Some((valor_marcado, texto)) = marcada(campos.unico("selTipoProcedimento")?)? else {
        return Err(ErroPropriedades::TipoDoProcessoNaoMarcado);
    };
    if let Some(oculto) = campos.opcional("hdnIdTipoProcedimento")?
        && valor(oculto) != valor_marcado
    {
        return Err(ErroPropriedades::TipoDoProcessoInconsistente);
    }
    Ok((valor_marcado, texto))
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
            id: valor,
            codigo: codigo.to_owned(),
            descricao: descricao.to_owned(),
        });
    }
    Ok(lidos)
}

fn interessados(campos: &Campos<'_>) -> Result<Vec<Interessado>, ErroPropriedades> {
    let select = campos.unico("selInteressadosProcedimento")?;
    opcoes(select)
        .into_iter()
        .map(|(id, _, texto)| {
            if texto.is_empty() {
                Err(ErroPropriedades::InteressadoForaDaFormaObservada)
            } else {
                Ok(Interessado { id, nome: texto })
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

// Derivação da alteração de especificação: hipótese ainda sem replay original.
pub(super) const ALTERAR_ESPECIFICACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: crate::acao_do_sei::AcaoDoSei::ProcedimentoAlterar,
    formulario: FORMULARIO,
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[
        "hdnFlagProcedimentoCadastro",
        "hdnIdProcedimento",
        "hdnProtocoloFormatado",
        "hdnIdTipoProcedimento",
        "hdnAssuntos",
        "hdnInteressadosProcedimento",
    ],
    campos_ocultos_reescritos: &[
        "hdnFlagProcedimentoCadastro",
        "hdnAssuntos",
        "hdnInteressadosProcedimento",
    ],
    campos_usuario: &[
        "selTipoProcedimento",
        "txtDescricao",
        "txtAssunto",
        "selAssuntos",
        "txtInteressadoProcedimento",
        "selInteressadosProcedimento",
        "txtInteressadoUsuario",
        "txaObservacoes",
        "selGrauSigilo",
        "rdoNivelAcesso",
        "selHipoteseLegal",
    ],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::CadastroAlterado,
    },
};

/// A propriedade de domínio None não distingue select vazio de option null.
/// Preservar a presença da opção servida em vez de sintetizar "null".
fn pares_select(
    campos: &Campos<'_>,
    nome: &'static str,
    pares: &mut Vec<(&'static str, String)>,
) -> Result<(), ErroPropriedades> {
    if let Some(select) = campos.opcional(nome)? {
        if select.presente("disabled") {
            return Ok(());
        }
        let opcoes = opcoes(select);
        let mut marcadas = opcoes.iter().filter(|(_, marcada, _)| *marcada);
        if select.presente("multiple") {
            pares.extend(marcadas.map(|(id, _, _)| (nome, id.clone())));
        } else if let Some((id, _, _)) = marcadas.next_back().or_else(|| opcoes.first()) {
            pares.push((nome, id.clone()));
        }
    }
    Ok(())
}

/// Reusar o leitor local de Campos, opcoes e observacoes, e o serializer
/// `valor_da_lupa` existente. A ordem final cabe a `derivar_submissao`.
pub(super) fn pares_de_especificacao(
    html: &crate::tela_html::HtmlServido,
    pedida: &str,
) -> Result<Vec<(&'static str, String)>, ErroPropriedades> {
    let dom = crate::tela_html::arvore(html);
    let formulario = match dom.com_id(FORMULARIO).collect::<Vec<_>>().as_slice() {
        [] => return Err(ErroPropriedades::FormularioDasPropriedadesAusente),
        [unico] => *unico,
        _ => return Err(ErroPropriedades::FormulariosDasPropriedadesRepetidos),
    };
    let campos = &Campos { formulario };
    if valor(campos.unico("hdnSinIndividual")?) != "N" {
        return Err(ErroPropriedades::CadastroIndividualNaoSuportado);
    }
    if nivel_de_acesso(campos)? == NivelDeAcesso::Sigiloso {
        return Err(ErroPropriedades::CadastroSigilosoNaoSuportado);
    }
    let mut pares = vec![
        ("txtDescricao", pedida.to_owned()),
        ("txaObservacoes", observacoes(campos)?),
        ("hdnFlagProcedimentoCadastro", "2".to_owned()),
    ];
    for nome in [
        "txtAssunto",
        "txtInteressadoProcedimento",
        "txtInteressadoUsuario",
    ] {
        if let Some(tag) = campos.opcional(nome)?
            && !tag.presente("disabled")
        {
            pares.push((nome, valor(tag)));
        }
    }
    for nome in [
        "selTipoProcedimento",
        "selAssuntos",
        "selInteressadosProcedimento",
        "selGrauSigilo",
        "selHipoteseLegal",
    ] {
        pares_select(campos, nome, &mut pares)?;
    }
    let nivel = match nivel_de_acesso(campos)? {
        NivelDeAcesso::Publico => "0",
        NivelDeAcesso::Restrito => "1",
        NivelDeAcesso::Sigiloso => "2",
    };
    pares.push(("rdoNivelAcesso", nivel.to_owned()));
    for (select, oculto) in [
        ("selAssuntos", "hdnAssuntos"),
        ("selInteressadosProcedimento", "hdnInteressadosProcedimento"),
    ] {
        // A projeção de consulta normaliza espaços; a derivação candidata
        // conserva o texto DOM. A semântica do JavaScript original ainda
        // precisa ser confrontada antes de habilitar a escrita.
        let itens: Vec<_> = campos
            .unico(select)?
            .elementos("option")
            .map(|no| (no.atributo("value").unwrap_or_default(), no.texto()))
            .collect();
        pares.push((
            oculto,
            valor_da_lupa(itens.iter().map(|(id, rotulo)| (*id, rotulo.as_str()))),
        ));
    }
    Ok(pares)
}

// Testes sintéticos da derivação, sem acesso remoto.
#[cfg(test)]
#[path = "tests_derivacao.rs"]
mod testes_derivacao;
