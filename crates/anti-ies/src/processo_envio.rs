//! Hipótese funcional de envio e anotação com estado derivado em SQLite.
//! Não representa a identidade exata do HTML de nenhuma instalação real.
use crate::{Estado, Resposta, despachar};
use iesei::instalacao::Instalacao;
use iesei::processos::{Falha, PedidoEnvio, ProcessoSimulado, enviar, ler, limpar_anotacao};
use rusqlite::Connection;
use std::fmt::Write;

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

fn pagina_de_envio(processo: &ProcessoSimulado) -> String {
    let mut pagina = format!(
        "<html><head><script type='text/javascript'>\n         objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}');\n         return 'palavras_pesquisa='+document.getElementById('txtUnidade').value+'&id_orgao='+document.getElementById('selOrgao').value;\n         </script></head><body>",
        "2".repeat(64)
    );
    write!(
        pagina,
        "<form id='frmAtividadeListar' method='post' action='controlador.php?acao=procedimento_enviar&acao_origem=procedimento_enviar&arvore=1&id_procedimento={}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}'>         <input type='hidden' name='hdnInfraTipoPagina' value='2'>         <button type='submit' name='sbmEnviar' value='Enviar'>Enviar</button>         <select id='selProcedimentos' name='selProcedimentos' multiple>         <option value='{}'>{} - Processo simulado</option></select>         <select id='selOrgao' name='selOrgao'>         <option value='null' selected>&nbsp;</option>         <option value='' selected>Todos</option></select>         <input type='text' id='txtUnidade' name='txtUnidade'>         <input type='hidden' name='hdnIdUnidade' value=''>         <select id='selUnidades' name='selUnidades' multiple></select>         <input type='checkbox' name='chkSinManterAberto'>         <input type='checkbox' name='chkSinRemoverAnotacoes'>         <input type='checkbox' name='chkSinEnviarEmailNotificacao'>         <input type='radio' name='rdoPrazo' value='1'>         <input type='text' name='txtPrazo' value=''>         <input type='radio' name='rdoPrazo' value='2'>         <input type='text' name='txtDias' value=''>         <input type='checkbox' name='chkSinDiasUteis'>         <input type='hidden' name='hdnIdProtocolos' value='{}'>         <input type='hidden' name='hdnIdAtividades' value='1'>         <input type='hidden' name='hdnUnidades' value=''>         </form></body></html>",
        html(&processo.id),
        "3".repeat(64),
        html(&processo.id),
        html(&processo.numero),
        html(&processo.id),
    )
    .expect("formatação de String");
    pagina
}

fn pagina_anotacao(processo: &ProcessoSimulado) -> Option<String> {
    let texto = processo.anotacao_origem.as_deref()?;
    Some(format!(
        "<html><body><form id='frmAnotacaoCadastro' method='post'          action='controlador.php?acao=anotacao_registrar&id_procedimento={}'>         <input type='hidden' name='hdnIdProtocolo' value='{}'>         <textarea id='txaDescricao' name='txaDescricao'>{}</textarea>         <button type='submit' name='sbmRegistrarAnotacao'>Salvar</button>         </form></body></html>",
        html(&processo.id),
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

fn destino_indicado(campos: &[(&str, &str)]) -> Option<String> {
    let unidades = campo_unico(campos, "hdnUnidades")?;
    let (id, _) = unidades
        .split_once("%B1")
        .or_else(|| unidades.split_once("%C2%B1"))?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.to_owned())
}

fn resposta_html(corpo: String) -> Resposta {
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo,
        localizacao: None,
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
    if metodo == "GET" {
        return match acao {
            "procedimento_enviar" => resposta_html(pagina_de_envio(&processo)),
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
        destino_indicado(&campos),
        checar_checkbox(&campos, "chkSinManterAberto"),
        checar_checkbox(&campos, "chkSinRemoverAnotacoes"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match enviar(
        banco,
        instalacao,
        cenario,
        PedidoEnvio {
            id,
            id_destino: &id_destino,
            manter_aberto,
            remover_anotacoes,
        },
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
    }
}

/// Decodifica o corpo de consulta form-urlencoded, inclusive escape percentual.
fn decodificar_formulario(texto: &str) -> Option<String> {
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

/// Projeta abertura de unidades na forma do nó raiz interpretada pelo IES.
/// Hipótese local revisável, com estado derivado exclusivamente do SQLite.
fn situacao_da_raiz(processo: &ProcessoSimulado) -> String {
    let ancora = |sigla: &str| -> String {
        let sigla = html(sigla);
        format!(r#"<a alt="{sigla}" title="{sigla}" class="ancoraSigla">{sigla}</a>"#)
    };
    match processo.aberto_em.as_slice() {
        [] => "Processo não possui andamentos abertos.<br />".to_owned(),
        [unidade] => format!(
            "Processo aberto somente na unidade {}.<br />",
            ancora(unidade)
        ),
        unidades => {
            let mut texto = String::from("Processo aberto nas unidades:<br />");
            for unidade in unidades {
                texto.push_str(&ancora(unidade));
                texto.push_str("<br />");
            }
            texto.push_str("<br />");
            texto
        }
    }
}

/// O leitor do IES interpreta literais JS, não executa a árvore.
fn literal_js(texto: &str) -> String {
    texto
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn arvore_do_processo(processo: &ProcessoSimulado) -> String {
    let id = &processo.id;
    let numero = literal_js(&processo.numero);
    let situacao = literal_js(&situacao_da_raiz(processo));
    format!(
        r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{id}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={id}","ifrVisualizacao","{numero}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{numero}");
Nos[0].acoes = '';
Nos[0].src = '';
Nos[0].html = '{situacao}';
</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#
    )
}

fn trabalho_do_processo(processo: &ProcessoSimulado) -> String {
    let id = &processo.id;
    format!(
        "<html><head><title>SEI - Processo</title></head><body>\
         <a id='lnkInfraUnidade'>{}</a>\
         <iframe id='ifrArvore' name='ifrArvore' \
         src='controlador.php?acao=procedimento_visualizar&amp;id_procedimento={id}'></iframe>\
         <iframe id='ifrVisualizacao' src='about:blank'></iframe></body></html>",
        html(&processo.unidade_origem)
    )
}

/// Consulta a árvore ou a tela de trabalho sem efeito; ausência do processo
/// delega à rota de documentos existente do mesmo cenário.
#[must_use]
pub fn navegacao(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if !super::eh_controlador_sei(alvo) {
        return None;
    }
    let acao = unico(alvo, "acao")?;
    if !matches!(acao, "procedimento_trabalhar" | "procedimento_visualizar") {
        return None;
    }
    let id = unico(alvo, "id_procedimento").filter(|id| id.bytes().all(|b| b.is_ascii_digit()))?;
    let processo = match ler(banco, instalacao, cenario, id) {
        Ok(Some(processo)) => processo,
        Ok(None) => return None,
        Err(_) => return Some(Resposta::nova(Estado::DadosInsuficientes)),
    };
    if metodo != "GET" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    Some(resposta_html(if acao == "procedimento_trabalhar" {
        trabalho_do_processo(&processo)
    } else {
        arvore_do_processo(&processo)
    }))
}
