//! Sobrestar e retomar no anti-IES, com estado SQLite e HTML derivado.
use crate::{Estado, Resposta, eh_controlador_sei, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::processos;
use iesei::sobrestamentos::{self, ErroSobrestamentos};
use rusqlite::Connection;

fn ambiente<'a>(
    alvo: &'a str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<(&'a str, String), Estado> {
    let id = parametro_unico(alvo, "id_procedimento")
        .filter(|v| v.bytes().all(|c| c.is_ascii_digit()))
        .ok_or(Estado::RequisicaoInvalida)?;
    let processo = processos::ler(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .ok_or(Estado::DadosInsuficientes)?;
    let unidade = crate::processo_envio::unidade_da_sessao(banco, instalacao, cenario, &processo)?;
    Ok((id, unidade))
}

fn pagina(id: &str) -> String {
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    format!(
        r#"<html><body>
<form id="frmDesentranharDocumento" method="post" action="controlador.php?acao=procedimento_sobrestar&amp;acao_origem=procedimento_sobrestar&amp;id_procedimento={id}&amp;infra_hash={hash}">
<input type="hidden" name="hdnInfraTipoPagina" value="2" />
<button type="submit" name="sbmSalvar" id="sbmSalvar" value="Salvar">Salvar</button>
<button type="button" name="btnCancelar" value="Cancelar">Cancelar</button>
<select id="selProcedimentos" name="selProcedimentos" multiple="multiple"><option value="{id}">Sintético</option></select>
<input type="radio" name="rdoSobrestar" id="optSomenteSobrestar" value="S" />
<input type="radio" name="rdoSobrestar" id="optSobrestarVincular" value="V" />
<input type="text" name="txtProcedimentoDestino" value="" />
<button type="button" name="btnPesquisar">Pesquisar</button>
<input type="text" name="txtIdentificacaoProcedimentoDestino" readonly="readonly" value="" />
<textarea id="txaMotivo" name="txaMotivo"></textarea>
<input type="hidden" name="hdnIdProtocolos" value="{id}" />
<input type="hidden" name="hdnIdProcedimentoDestino" value="" />
</form></body></html>"#
    )
}

fn erro(erro: &ErroSobrestamentos) -> Estado {
    match erro {
        ErroSobrestamentos::ProcessoNaoEncontrado
        | ErroSobrestamentos::Sqlite(_)
        | ErroSobrestamentos::Processo(_) => Estado::DadosInsuficientes,
        ErroSobrestamentos::UnidadeNaoAberta
        | ErroSobrestamentos::JaSobrestado
        | ErroSobrestamentos::NaoSobrestado
        | ErroSobrestamentos::OutraUnidade
        | ErroSobrestamentos::DadoInvalido => Estado::RequisicaoInvalida,
    }
}

/// O GET da tela do nó publica a URL da função de retomada, sem mutar.
pub(crate) fn tela(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if !eh_controlador_sei(alvo)
        || parametro_unico(alvo, "acao") != Some("arvore_visualizar")
        || parametro_unico(alvo, "id_documento").is_some()
    {
        return None;
    }
    let id = parametro_unico(alvo, "id_procedimento")?;
    let estado = match sobrestamentos::consultar(banco, instalacao, cenario, id) {
        Ok(Some(s)) => s,
        Ok(None) => return None,
        Err(_) => return Some(Resposta::nova(Estado::DadosInsuficientes)),
    };
    if metodo != "GET" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    let Ok((_, unidade)) = ambiente(alvo, banco, instalacao, cenario) else {
        return Some(Resposta::nova(Estado::DadosInsuficientes));
    };
    if estado.unidade != unidade {
        return Some(Resposta::nova(Estado::RequisicaoInvalida));
    }
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    Some(Resposta::html(format!(
        "<html><head><script>\nvar linkRemoverSobrestamentoProcesso = 'controlador.php?acao=procedimento_remover_sobrestamento&id_procedimento={id}&infra_hash={hash}';\n</script></head><body></body></html>"
    )))
}

/// Uma única escrita por POST sobrestar ou GET remover, sem retry.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    let Some(acao @ ("procedimento_sobrestar" | "procedimento_remover_sobrestamento")) =
        parametro_unico(alvo, "acao")
    else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let (id, unidade) = match ambiente(alvo, banco, instalacao, cenario) {
        Ok(v) => v,
        Err(estado) => return Resposta::nova(estado),
    };
    if acao == "procedimento_remover_sobrestamento" {
        if metodo != "GET" {
            return Resposta::nova(Estado::MetodoNaoPermitido);
        }
        return match sobrestamentos::retomar(banco, instalacao, cenario, id, &unidade) {
            Ok(()) => Resposta::html("<html><body>Retomado</body></html>".to_owned()),
            Err(e) => Resposta::nova(erro(&e)),
        };
    }
    if metodo == "GET" {
        return match sobrestamentos::consultar(banco, instalacao, cenario, id) {
            Ok(None) => Resposta::html(pagina(id)),
            Ok(Some(_)) => Resposta::nova(Estado::RequisicaoInvalida),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(pares) = crate::processo_envio::pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let campo = |nome| crate::processo_envio::campo_unico(&pares, nome);
    let Some(motivo) = campo("txaMotivo").and_then(crate::processo_envio::decodificar_formulario)
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if campo("hdnIdProtocolos") != Some(id)
        || campo("rdoSobrestar") != Some("S")
        || campo("hdnInfraTipoPagina") != Some("2")
        || campo("hdnIdProcedimentoDestino") != Some("")
        || campo("txtProcedimentoDestino") != Some("")
        || campo("txtIdentificacaoProcedimentoDestino") != Some("")
        || campo("sbmSalvar") != Some("Salvar")
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match sobrestamentos::sobrestar(banco, instalacao, cenario, id, &unidade, &motivo) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_sobrestar&id_procedimento={id}"
        )),
        Err(e) => Resposta::nova(erro(&e)),
    }
}
