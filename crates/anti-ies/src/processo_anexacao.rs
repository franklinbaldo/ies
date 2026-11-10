//! Formulário de anexação em estado SQLite, somente em bancada sintética.
//! Hipótese declarada em Todos/src/todos/anexacao.py @323eb01.
use crate::{Estado, Resposta, eh_controlador_sei, escapar, parametro_unico};
use iesei::{
    anexacoes,
    editor_documento::{self, SituacaoAssinaturas},
    instalacao::Instalacao,
    processos,
};
use rusqlite::Connection;
use std::fmt::Write as _;

fn contexto<'a>(
    alvo: &'a str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Result<&'a str, Estado> {
    let id = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(Estado::RequisicaoInvalida)?;
    processos::ler(banco, origem, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .ok_or(Estado::DadosInsuficientes)?;
    Ok(id)
}

fn formulario(id: &str, anexados: &[anexacoes::Anexado]) -> Resposta {
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let mut linhas = String::new();
    for a in anexados {
        write!(
            linhas,
            "<tr><td><input type=\"checkbox\" value=\"{}\" /></td><td>{}</td></tr>",
            escapar(&a.id),
            escapar(&a.numero)
        )
        .expect("formatacao String");
    }
    let tabela = if anexados.is_empty() {
        String::new()
    } else {
        format!("<table summary=\"Lista de Processos Anexados\">{linhas}</table>")
    };
    Resposta::html(format!(
        r#"<html><head><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={hash}');
objVerifica = new infraAjaxComplementar(null,'controlador_ajax.php?acao_ajax=procedimento_anexacao_verificar&infra_hash={hash}');
</script></head><body>
<form id="frmProcedimentoAnexar" method="post" action="controlador.php?acao=procedimento_anexar&id_procedimento={id}&infra_hash={hash}">
<input name="txtProtocolo" type="text" value="" />
<input name="txtIdentificacaoProtocolo" type="text" value="" />
<input name="hdnIdProtocolo" type="hidden" value="" />
<input name="hdnAnexacao" type="hidden" value="" />
<input name="sbmAnexar" type="submit" value="Anexar" />
</form>{tabela}</body></html>"#
    ))
}

/// Consulta de identidade quando há processos, mas nenhuma relação observada.
/// Não interfere na rota de relacionamentos que já usa o mesmo RI1023.
pub(crate) fn identificar(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if parametro_unico(alvo, "acao_ajax") != Some("protocolo_RI1023")
        || !alvo
            .split_once('?')
            .map_or(alvo, |(c, _)| c)
            .ends_with("/controlador_ajax.php")
    {
        return None;
    }
    if matches!(
        iesei::relacionamentos::existe_contexto(banco, origem, cenario),
        Ok(true)
    ) {
        return None;
    }
    if metodo != "POST" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    let Some(pares) = crate::processo_envio::pares(corpo) else {
        return Some(Resposta::nova(Estado::RequisicaoInvalida));
    };
    let Some(nup) = crate::processo_envio::campo_unico(&pares, "idProcedimento")
        .and_then(crate::processo_envio::decodificar_formulario)
    else {
        return Some(Resposta::nova(Estado::RequisicaoInvalida));
    };
    let corpo = match iesei::relacionamentos::identificar(banco, origem, cenario, &nup) {
        Ok(Some(destino)) => format!(
            "<complementos><complemento nome=\"IdProcedimento\">{}</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{}</complemento><complemento nome=\"NomeTipoProcedimento\">Processo Sintético</complemento></complementos>",
            escapar(&destino.id),
            escapar(&destino.numero)
        ),
        Ok(None) => "<erros><erro>Processo não observado</erro></erros>".into(),
        Err(_) => return Some(Resposta::nova(Estado::DadosInsuficientes)),
    };
    Some(Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "application/xml; charset=utf-8",
        corpo,
        localizacao: None,
        corpo_binario: None,
    })
}

/// Consulta de assinaturas publicada como AJAX. A resposta deriva do estado
/// documental sintético, nunca da leitura de uma assinatura real do SEI.
pub(crate) fn consulta(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if parametro_unico(alvo, "acao_ajax") != Some("procedimento_anexacao_verificar") {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = crate::processo_envio::pares(corpo)
        .and_then(|pares| {
            crate::processo_envio::campo_unico(&pares, "idProcedimento").map(str::to_owned)
        })
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if !matches!(processos::ler(banco, origem, cenario, &id), Ok(Some(_))) {
        return Resposta::nova(Estado::DadosInsuficientes);
    }
    let Ok(situacao) = editor_documento::situacao_assinaturas_anexacao(banco, origem, cenario, &id)
    else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let resultado = match situacao {
        SituacaoAssinaturas::SemPendencias => "S",
        SituacaoAssinaturas::Pendentes => "N",
        SituacaoAssinaturas::NaoObservadas => {
            return Resposta::nova(Estado::DadosInsuficientes);
        }
    };
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "application/xml; charset=utf-8",
        corpo: format!(
            "<complementos><complemento nome=\"Anexacao\">{resultado}</complemento></complementos>"
        ),
        localizacao: None,
        corpo_binario: None,
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
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("procedimento_anexar") {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    let id = match contexto(alvo, banco, origem, cenario) {
        Ok(id) => id,
        Err(estado) => return Resposta::nova(estado),
    };
    if metodo == "GET" {
        return match anexacoes::listar(banco, origem, cenario, id) {
            Ok(Some(anexados)) => formulario(id, &anexados),
            _ => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(pares) = crate::processo_envio::pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let campo = |nome| crate::processo_envio::campo_unico(&pares, nome);
    let (Some(destino), Some(nup)) = (campo("hdnIdProtocolo"), campo("txtProtocolo")) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if campo("hdnAnexacao") != Some("S")
        || campo("sbmAnexar") != Some("Anexar")
        || campo("txtIdentificacaoProtocolo").is_none()
        || !destino.bytes().all(|b| b.is_ascii_digit())
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(Some(processo)) = processos::ler(banco, origem, cenario, destino) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if crate::processo_envio::decodificar_formulario(nup).as_deref() != Some(&processo.numero) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match anexacoes::anexar(banco, origem, cenario, id, destino) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_anexar&resultado=1&id_procedimento={id}"
        )),
        Err(anexacoes::ErroAnexacoes::DadoInvalido | anexacoes::ErroAnexacoes::JaAnexado) => {
            Resposta::nova(Estado::RequisicaoInvalida)
        }
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
