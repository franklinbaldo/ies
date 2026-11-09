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
            | Err(relacionamentos::Falha::DadoInvalido | relacionamentos::Falha::DestinoAmbiguo) => {
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
            Err(relacionamentos::Falha::DadoInvalido) => Resposta::nova(Estado::RequisicaoInvalida),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
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

#[cfg(test)]
mod testes {
    use super::*;
    use crate::rotas;
    use iesei::banco_cassettes::iniciar;
    use iesei::processos::{ProcessoSimulado, guardar};

    fn cenario() -> (Connection, Instalacao) {
        let mut db = Connection::open_in_memory().unwrap();
        iniciar(&db).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        for (id, numero) in [("10", "P-01"), ("22", "P-02")] {
            guardar(
                &mut db,
                &origem,
                "caso",
                &ProcessoSimulado {
                    id: id.into(),
                    numero: numero.into(),
                    unidade_origem: "TESTE".into(),
                    aberto_em: vec!["TESTE".into()],
                    anotacao_origem: None,
                    destinos: vec![],
                },
            )
            .unwrap();
            relacionamentos::observar(&db, &origem, "caso", id).unwrap();
        }
        (db, origem)
    }

    /// Apoio ao BDD-258-02: o POST sintético só é aceito quando uma
    /// nova GET, inclusive pelo processo inverso, lê o vínculo no SQLite.
    #[test]
    fn adicionar_post_reler_bilateralmente_e_preservar_estado() {
        let (db, origem) = cenario();
        let url = "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10";
        let inicial = rotas::responder("GET", url, b"", &db, &origem, "caso");
        assert_eq!(inicial.estado, Estado::Ok);
        assert!(!inicial.corpo.contains("P-02</a>"));

        let resposta = rotas::responder(
            "POST",
            url,
            b"hdnIdProtocolo=22&txtProtocolo=P-02&txtIdentificacaoProtocolo=&sbmAdicionar=Adicionar",
            &db,
            &origem,
            "caso",
        );
        assert_eq!(resposta.estado, Estado::Redirecionamento);
        assert!(
            resposta
                .localizacao
                .as_deref()
                .is_some_and(|s| s.contains("resultado=1"))
        );
        let depois = rotas::responder("GET", url, b"", &db, &origem, "caso");
        assert_eq!(depois.estado, Estado::Ok);
        assert!(depois.corpo.contains("P-02</a>"));
        let inverso = rotas::responder(
            "GET",
            "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=22",
            b"",
            &db,
            &origem,
            "caso",
        );
        assert!(inverso.corpo.contains("P-01</a>"));
        assert_eq!(
            relacionamentos::listar(&db, &origem, "caso", "10")
                .unwrap()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            relacionamentos::listar(&db, &origem, "caso", "22")
                .unwrap()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn adicionar_recusa_identidade_que_nao_corresponde_ao_nup() {
        let (db, origem) = cenario();
        let url = "/sei/controlador.php?acao=procedimento_relacionar&id_procedimento=10";
        let resposta = rotas::responder(
            "POST",
            url,
            b"hdnIdProtocolo=22&txtProtocolo=P-03&txtIdentificacaoProtocolo=&sbmAdicionar=Adicionar",
            &db,
            &origem,
            "caso",
        );
        assert_eq!(resposta.estado, Estado::RequisicaoInvalida);
        assert_eq!(
            relacionamentos::listar(&db, &origem, "caso", "10")
                .unwrap()
                .unwrap(),
            vec![]
        );
    }
}
