//! Prévia do formulário de relacionamentos, derivada do SQLite versionado.
//! Imita a forma PEN1 apenas como hipótese sintética; sem escrita remota.
use crate::{Estado, Resposta, eh_controlador_sei, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::relacionamentos;
use rusqlite::Connection;
use std::fmt::Write as _;

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(vinculos)) = relacionamentos::listar(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let assinatura = "b".repeat(64);
    let mut linhas = String::new();
    for vinculo in vinculos {
        write!(
            linhas,
            "<tr><td><a href=\"controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={}&amp;infra_hash={assinatura}\">{}</a></td></tr>",
            escapar(&vinculo.id), escapar(&vinculo.numero),
        ).expect("String");
    }
    Resposta::html(format!(
        r#"<html><head><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={assinatura}');
</script></head><body>
<form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&id_procedimento={id}&infra_hash={assinatura}">
<input type="text" id="txtProtocolo" name="txtProtocolo" value=""/>
<input type="text" id="txtIdentificacaoProtocolo" name="txtIdentificacaoProtocolo" value=""/>
<input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value=""/>
<input type="hidden" id="hdnInfraItemId" name="hdnInfraItemId" value=""/>
<button id="sbmAdicionar" name="sbmAdicionar" type="submit">Adicionar</button>
<table summary="Lista de Processos Relacionados"><tr><th>Processo</th></tr>{linhas}</table>
</form></body></html>"#
    ))
}
fn decodificar(texto: &str) -> Option<String> {
    let bytes = texto.as_bytes();
    let mut saida = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => saida.push(b' '),
            b'%' => {
                let h = char::from(*bytes.get(i + 1)?).to_digit(16)?;
                let l = char::from(*bytes.get(i + 2)?).to_digit(16)?;
                saida.push(u8::try_from(h * 16 + l).ok()?);
                i += 2;
            }
            b => saida.push(b),
        }
        i += 1;
    }
    String::from_utf8(saida).ok()
}
pub(crate) fn consulta(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !alvo
        .split_once('?')
        .map_or(alvo, |(c, _)| c)
        .ends_with("/controlador_ajax.php")
    {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Ok(true) = relacionamentos::existe_contexto(banco, origem, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let mut valores = corpo
        .split('&')
        .filter_map(|p| p.split_once('='))
        .filter(|(chave, _)| *chave == "idProcedimento")
        .map(|(_, valor)| valor);
    let Some(pedido) = valores
        .next()
        .and_then(decodificar)
        .filter(|v| !v.is_empty())
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if valores.next().is_some() {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(destino) = relacionamentos::identificar(banco, origem, cenario, &pedido) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let corpo = if let Some(destino) = destino {
        format!(
            "<complementos><complemento nome=\"IdProcedimento\">{}</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{}</complemento></complementos>",
            escapar(&destino.id),
            escapar(&destino.numero),
        )
    } else {
        "<erros><erro>Destino não observado</erro></erros>".to_owned()
    };
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "application/xml; charset=utf-8",
        corpo,
        localizacao: None,
        corpo_binario: None,
    }
}
