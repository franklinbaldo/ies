//! Prévia do formulário de relacionamentos, derivada do SQLite versionado.
//! Imita a forma PEN1 apenas como hipótese sintética; sem escrita remota.
use crate::{
    Estado, Resposta, documentos::campo_formulario_unico, eh_controlador_sei, escapar,
    parametro_unico,
};
use iesei::instalacao::Instalacao;
use iesei::relacionamentos;
use rusqlite::Connection;
use std::fmt::Write as _;

/// Uma exclusão retira somente a aresta observada, sem inventar destino.
fn responder_remocao(
    corpo: &str,
    id: &str,
    vinculos: &[relacionamentos::Relacionamento],
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Some(id_destino) = campo_formulario_unico(corpo, "hdnInfraItemId") else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if id_destino.is_empty()
        || !id_destino.bytes().all(|b| b.is_ascii_digit())
        || !vinculos.iter().any(|r| r.id == id_destino)
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match relacionamentos::remover(banco, origem, cenario, id, &id_destino) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_relacionar&resultado=1&id_procedimento={id}"
        )),
        Err(
            relacionamentos::ErroRelacionamentos::DadoInvalido
            | relacionamentos::ErroRelacionamentos::ProcessoNaoObservado,
        ) => Resposta::nova(Estado::RequisicaoInvalida),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if !matches!(metodo, "GET" | "POST") {
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
    if parametro_unico(alvo, "acao") == Some("procedimento_excluir_relacionamento") {
        if metodo != "POST" {
            return Resposta::nova(Estado::MetodoNaoPermitido);
        }
        return responder_remocao(corpo, id, &vinculos, banco, origem, cenario);
    }
    if metodo == "POST" {
        // A identidade oferecida pela consulta tem de coincidir com o id
        // submetido: o navegador não escolhe um processo só pelo campo oculto.
        let (Some(id_destino), Some(numero), Some(_tipo), Some(_botao)) = (
            campo_formulario_unico(corpo, "hdnIdProtocolo"),
            campo_formulario_unico(corpo, "txtProtocolo"),
            campo_formulario_unico(corpo, "txtIdentificacaoProtocolo"),
            campo_formulario_unico(corpo, "sbmAdicionar"),
        ) else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        if !id_destino.bytes().all(|b| b.is_ascii_digit())
            || id_destino.is_empty()
            || id_destino == id
        {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        let destino = match relacionamentos::identificar(banco, origem, cenario, &numero) {
            Ok(Some(destino)) => destino,
            Ok(None)
            | Err(
                relacionamentos::ErroRelacionamentos::DadoInvalido
                | relacionamentos::ErroRelacionamentos::DestinoAmbiguo,
            ) => {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
        };
        if destino.id != id_destino || vinculos.iter().any(|r| r.id == id_destino) {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        return match relacionamentos::registrar(banco, origem, cenario, id, &id_destino) {
            Ok(()) => Resposta::redirecionar(format!(
                "controlador.php?acao=procedimento_relacionar&resultado=1&id_procedimento={id}"
            )),
            Err(relacionamentos::ErroRelacionamentos::DadoInvalido) => {
                Resposta::nova(Estado::RequisicaoInvalida)
            }
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
    renderizar_formulario(id, &vinculos)
}

/// Projeta a lista observada no formulário PEN1; lista vazia não inventa tabela.
fn renderizar_formulario(id: &str, vinculos: &[relacionamentos::Relacionamento]) -> Resposta {
    let assinatura = "b".repeat(64);
    let tem_vinculos = !vinculos.is_empty();
    let mut linhas = String::new();
    for vinculo in vinculos {
        write!(
            linhas,
            "<tr><td><a href=\"controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={}&amp;infra_hash={assinatura}\">{}</a></td><td><a href=\"#ID-{}\" onclick=\"acaoRemover('{}');\">Remover</a></td></tr>",
            escapar(&vinculo.id),
            escapar(&vinculo.numero),
            escapar(&vinculo.id),
            escapar(&vinculo.id),
        ).expect("String");
    }
    // A captura PEN1 publica formulário vazio, mas não tabela nem ação
    // de remoção, após excluir a última relação observada.
    let acao_remover = if tem_vinculos {
        format!(
            r#"function acaoRemover(id){{
  if (confirm("Confirma exclusão do relacionamento?")){{
    document.getElementById('hdnInfraItemId').value=id;
    document.getElementById('frmProcedimentoListarRelacionamentos').action='controlador.php?acao=procedimento_excluir_relacionamento&acao_origem=procedimento_relacionar&arvore=1&id_procedimento={id}&infra_hash={assinatura}';
    document.getElementById('frmProcedimentoListarRelacionamentos').submit();
  }}
}}"#
        )
    } else {
        String::new()
    };
    let campo_remover = if tem_vinculos {
        r#"<input type="hidden" id="hdnInfraItemId" name="hdnInfraItemId" value=""/>"#
    } else {
        ""
    };
    let tabela = if tem_vinculos {
        format!(
            r#"<table summary="Lista de Processos Relacionados"><tr><th>Processo</th></tr>{linhas}</table>"#
        )
    } else {
        String::new()
    };
    Resposta::html(format!(
        r#"<html><head><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={assinatura}');
{acao_remover}
</script></head><body>
<form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&id_procedimento={id}&infra_hash={assinatura}">
<input type="text" id="txtProtocolo" name="txtProtocolo" value=""/>
<input type="text" id="txtIdentificacaoProtocolo" name="txtIdentificacaoProtocolo" value=""/>
<input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value=""/>
{campo_remover}
<button id="sbmAdicionar" name="sbmAdicionar" type="submit">Adicionar</button>
{tabela}
</form></body></html>"#
    ))
}
fn decodificar(texto: &str) -> Option<String> {
    let bytes = texto.as_bytes();
    let mut saida = Vec::new();
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        match byte {
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

#[cfg(test)]
#[path = "processo_relacionar/testes.rs"]
mod testes;
