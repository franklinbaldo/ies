//! Hipótese funcional de envio e anotação com estado derivado em SQLite.
//! Não representa a identidade exata do HTML de nenhuma instalação real.
use crate::{Estado, Resposta, despachar};
use iesei::instalacao::Instalacao;
use iesei::processos::{
    Falha, PedidoEnvio, ProcessoSimulado, enviar_da_unidade, ler, limpar_anotacao,
};
use rusqlite::Connection;
use std::fmt::Write;

mod formulario;
mod navegacao;
use formulario::pagina_de_envio;
pub use navegacao::navegacao;

fn unico<'a>(alvo: &'a str, chave: &str) -> Option<&'a str> {
    let (_, query) = alvo.split_once('?')?;
    let mut valores = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .filter(|(k, _)| *k == chave)
        .map(|(_, v)| v);
    let valor = valores.next()?;
    (!valor.is_empty() && valores.next().is_none()).then_some(valor)
}

fn html(texto: &str) -> String {
    super::escapar(texto)
}

/// A unidade ativa vem da fotografia do Controle R; cenários antigos sem
/// Controle R conservam a origem do processo como unidade da sessão.
fn unidade_da_sessao(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    processo: &ProcessoSimulado,
) -> Result<String, Estado> {
    match iesei::banco_cassettes::ler(banco, instalacao, cenario) {
        Ok(Some(controle)) => Ok(controle.unidade.as_str().to_owned()),
        Ok(None) => Ok(processo.unidade_origem.clone()),
        Err(_) => Err(Estado::DadosInsuficientes),
    }
}

fn pagina_anotacao(processo: &ProcessoSimulado) -> Option<String> {
    let texto = processo.anotacao_origem.as_deref()?;
    Some(format!(
        "<html><body><form id='frmAnotacaoCadastro' method='post'          action='controlador.php?acao=anotacao_registrar&id_procedimento={}&infra_hash={}'>         <input type='hidden' name='hdnIdProtocolo' value='{}'>         <textarea id='txaDescricao' name='txaDescricao'>{}</textarea>         <button type='submit' name='sbmRegistrarAnotacao'>Salvar</button>         </form></body></html>",
        html(&processo.id),
        "a".repeat(64),
        html(&processo.id),
        html(texto),
    ))
}

fn pares(corpo: &str) -> Option<Vec<(&str, &str)>> {
    if corpo.len() > 16_384 {
        return None;
    }
    corpo
        .split('&')
        .map(|campo| campo.split_once('='))
        .collect::<Option<Vec<_>>>()
}

fn campo_unico<'a>(campos: &'a [(&str, &str)], chave: &str) -> Option<&'a str> {
    let mut valores = campos.iter().filter(|(k, _)| *k == chave).map(|(_, v)| *v);
    let valor = valores.next()?;
    valores.next().is_none().then_some(valor)
}

fn checar_checkbox(campos: &[(&str, &str)], chave: &str) -> Option<bool> {
    match campo_unico(campos, chave) {
        None if campos.iter().all(|(k, _)| *k != chave) => Some(false),
        Some("on") => Some(true),
        _ => None,
    }
}

fn destino_indicado(campos: &[(&str, &str)], processo: &ProcessoSimulado) -> Option<String> {
    let unidades = campo_unico(campos, "hdnUnidades")?;
    // O sufixo %B1 está contido em %C2%B1: testar UTF-8 primeiro evita
    // interpretar "2%C2" como identificador do destino.
    let (id, descricao) = unidades
        .split_once("%C2%B1")
        .or_else(|| unidades.split_once("%B1"))?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // A escolha efetiva é o par id/sigla publicado pelo autocomplete,
    // não apenas um id verdadeiro acompanhado de um rótulo inventado.
    // A descrição pode seguir a sigla ("SIGLA - Nome") ou ser só a sigla.
    let sigla = descricao
        .split_once("+-+")
        .or_else(|| descricao.split_once("%20-%20"))
        .map_or(descricao, |(sigla, _)| sigla);
    let sigla = decodificar_formulario(sigla)?;
    processo
        .destinos
        .iter()
        .any(|destino| destino.id == id && destino.sigla == sigla)
        .then(|| id.to_owned())
}

fn resposta_html(corpo: String) -> Resposta {
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo,
        localizacao: None,
        corpo_binario: None,
    }
}

fn redirecionado(acao: &str, id: &str) -> Resposta {
    Resposta {
        estado: Estado::Redirecionamento,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo: String::new(),
        localizacao: Some(format!(
            "controlador.php?acao=arvore_visualizar&acao_origem={acao}&id_procedimento={id}"
        )),
        corpo_binario: None,
    }
}

fn estado_da_falha(falha: &Falha) -> Estado {
    match falha {
        Falha::ProcessoNaoEncontrado | Falha::Sqlite(_) => Estado::DadosInsuficientes,
        Falha::DadoInvalido
        | Falha::OrigemNaoAberta
        | Falha::DestinoNaoOferecido
        | Falha::DestinoJaAberto
        | Falha::AnotacaoNaoObservada => Estado::RequisicaoInvalida,
    }
}

/// Serve a prévia do envio e executa uma transição única, verificável por GET.
///
/// # Cautela
/// É um simulador offline. Não confirma compatibilidade com uma instalação SEI.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !super::eh_controlador_sei(alvo) {
        return despachar(metodo, alvo);
    }
    let Some(acao @ ("procedimento_enviar" | "anotacao_registrar")) = unico(alvo, "acao") else {
        return despachar(metodo, alvo);
    };
    let Some(id) =
        unico(alvo, "id_procedimento").filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(processo)) = ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let unidade_atual = match unidade_da_sessao(banco, instalacao, cenario, &processo) {
        Ok(unidade) => unidade,
        Err(estado) => return Resposta::nova(estado),
    };
    // A anotacao continua legivel apos a saida da origem: o GET permite
    // verificar a remocao por releitura independente. Um POST novo, porem,
    // nao pode editar uma unidade que ja foi fechada pela tramitacao.
    if (acao == "procedimento_enviar" && !processo.aberto_em.contains(&unidade_atual))
        || (acao == "anotacao_registrar"
            && (unidade_atual != processo.unidade_origem
                || (metodo == "POST" && !processo.aberto_em.contains(&unidade_atual))))
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    if metodo == "GET" {
        return match acao {
            "procedimento_enviar" => resposta_html(pagina_de_envio(&processo, &unidade_atual)),
            "anotacao_registrar" => pagina_anotacao(&processo)
                .map_or_else(|| Resposta::nova(Estado::DadosInsuficientes), resposta_html),
            _ => unreachable!(),
        };
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(campos) = pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if acao == "anotacao_registrar" {
        if campo_unico(&campos, "hdnIdProtocolo") != Some(id)
            || campo_unico(&campos, "txaDescricao") != Some("")
        {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        return match limpar_anotacao(banco, instalacao, cenario, id) {
            Ok(()) => redirecionado(acao, id),
            Err(falha) => Resposta::nova(estado_da_falha(&falha)),
        };
    }
    if campo_unico(&campos, "hdnIdProtocolos") != Some(id) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let (Some(id_destino), Some(manter_aberto), Some(remover_anotacoes)) = (
        destino_indicado(&campos, &processo),
        checar_checkbox(&campos, "chkSinManterAberto"),
        checar_checkbox(&campos, "chkSinRemoverAnotacoes"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match enviar_da_unidade(
        banco,
        instalacao,
        cenario,
        PedidoEnvio {
            id,
            id_destino: &id_destino,
            manter_aberto,
            remover_anotacoes,
        },
        &unidade_atual,
    ) {
        Ok(_) => redirecionado(acao, id),
        Err(falha) => Resposta::nova(estado_da_falha(&falha)),
    }
}

/// Responde ao autocomplete publicado na tela de envio a partir do SQLite.
///
/// O pedido consulta a instalação/cenário, não depende de um HTML fixo nem
/// altera estado. A filtragem é uma hipótese revisável para o anti-IES.
#[must_use]
pub fn autocompletar(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let caminho = alvo.split_once('?').map_or(alvo, |(caminho, _)| caminho);
    if !caminho.ends_with("/controlador_ajax.php")
        || unico(alvo, "acao_ajax") != Some("unidade_auto_completar_envio_processo")
    {
        return despachar(metodo, alvo);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(campos) = pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Some(termo) = campo_unico(&campos, "palavras_pesquisa")
        .and_then(decodificar_formulario)
        .filter(|texto| !texto.trim().is_empty())
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if campo_unico(&campos, "id_orgao").is_none()
        || !matches!(campo_unico(&campos, "unidade_atual"), None | Some("0"))
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(Some(destinos)) = iesei::processos::destinos_do_cenario(banco, instalacao, cenario)
    else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let termo = termo.to_lowercase();
    let mut xml = String::from("<?xml version='1.0' encoding='UTF-8'?><itens>");
    for destino in destinos {
        if destino.descricao.to_lowercase().contains(&termo) {
            write!(
                xml,
                "<item id='{}' descricao='{}'></item>",
                html(&destino.id),
                html(&destino.descricao)
            )
            .expect("formatação String");
        }
    }
    xml.push_str("</itens>");
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "application/xml; charset=utf-8",
        corpo: xml,
        localizacao: None,
        corpo_binario: None,
    }
}

/// Decodifica o corpo de consulta form-urlencoded, inclusive escape percentual.
pub(crate) fn decodificar_formulario(texto: &str) -> Option<String> {
    let bytes = texto.as_bytes();
    let mut saida = Vec::with_capacity(bytes.len());
    let mut posicao = 0;
    while posicao < bytes.len() {
        match bytes[posicao] {
            b'+' => saida.push(b' '),
            b'%' => {
                let alto = (char::from(*bytes.get(posicao + 1)?)).to_digit(16)?;
                let baixo = (char::from(*bytes.get(posicao + 2)?)).to_digit(16)?;
                saida.push(u8::try_from((alto << 4) + baixo).ok()?);
                posicao += 2;
            }
            byte => saida.push(byte),
        }
        posicao += 1;
    }
    String::from_utf8(saida).ok()
}
