//! Composição sintética da consulta de grupo com o anti-IES de tramitação.
//! O catálogo de grupo é servido pelo proxy; processo e autocomplete vêm do SQLite.
use super::*;

fn grupo(ensaio: EnsaioRemocao) -> bool {
    matches!(
        ensaio,
        EnsaioRemocao::GrupoValido | EnsaioRemocao::GrupoIdDivergente
    )
}

pub(super) fn resposta(ensaio: EnsaioRemocao, metodo: &str, acao: &str) -> Option<Vec<u8>> {
    if !grupo(ensaio) || metodo != "GET" {
        return None;
    }
    let hash = "a".repeat(64);
    let id = if ensaio == EnsaioRemocao::GrupoIdDivergente {
        "002"
    } else {
        "2"
    };
    let html = match acao {
        "grupo_unidade_listar" => format!(
            "<form id='frmGrupoUnidadeLista'><table><caption>Grupos de Envio (1 registro):</caption>\
             <tr><th></th><th>Nome</th><th>Ações</th></tr>\
             <tr><td><input type='checkbox' value='004'></td><td>Grupo</td>\
             <td><a href='controlador.php?acao=grupo_unidade_consultar&amp;id_grupo_unidade=004&amp;infra_hash={hash}'>Consultar</a></td></tr>\
             </table></form>"
        ),
        "grupo_unidade_consultar" => format!(
            "<form id='frmGrupoCadastro'><input name='hdnIdGrupoUnidade' value='004'>\
             <input name='txtNome' value='Grupo'><textarea name='txaDescricao'>Teste</textarea>\
             <input name='hdnUnidades' value=''><select name='selUnidades'>\
             <option value='{id}'>DESTINO - Unidade de teste</option></select></form>"
        ),
        _ => return None,
    };
    Some(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
            html.len()
        )
        .into_bytes(),
    )
}

pub(super) fn ajustar_controle(
    resposta: Vec<u8>,
    ensaio: EnsaioRemocao,
    metodo: &str,
    acao: &str,
) -> Vec<u8> {
    if !grupo(ensaio) || metodo != "GET" || acao != "procedimento_controlar" {
        return resposta;
    }
    let texto = String::from_utf8(resposta).unwrap();
    let (cabecalho, corpo) = texto.split_once("\r\n\r\n").unwrap();
    let hash = "a".repeat(64);
    assert!(corpo.contains("</body>"));
    let corpo = corpo.replacen(
        "</body>",
        &format!(
            "<a href='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={hash}'>Grupos</a></body>"
        ),
        1,
    );
    let cabecalho = cabecalho
        .split("\r\n")
        .map(|linha| {
            if linha.to_ascii_lowercase().starts_with("content-length:") {
                format!("Content-Length: {}", corpo.len())
            } else {
                linha.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    format!("{cabecalho}\r\n\r\n{corpo}").into_bytes()
}

fn sem_mutacao(pedidos: &[String], depois: &ProcessoSimulado) {
    assert!(
        pedidos.iter().all(|p| {
            !p.starts_with("POST ")
                || matches!(
                    p.as_str(),
                    "POST protocolo_pesquisa_rapida" | "POST unidade_auto_completar_envio_processo"
                )
        }),
        "{pedidos:?}"
    );
    assert_eq!(depois.aberto_em, ["ORIGEM"]);
    assert_eq!(depois.anotacao_origem.as_deref(), Some("Nota da origem"));
}

#[test]
fn grupo_preserva_saida_unitaria_e_uniao_deduplicada_sem_escrever() {
    let grupo = ["--json", "processo", "enviar", NUP, "--grupo", "004"];
    let mut uniao = grupo.to_vec();
    uniao.extend(["--para", "DESTINO", "--para", "ARQUIVO", "--manter-aberto"]);
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&grupo, &uniao],
        false,
        Some("Nota da origem"),
        false,
        EnsaioRemocao::GrupoValido,
    );
    for saida in &saidas {
        assert_eq!(saida.status.code(), Some(0), "{saida:?}; {pedidos:?}");
        let valor = json(saida);
        assert_eq!(valor["grupo_id"], "004");
        assert_eq!(valor["confirmado"], false);
        assert_eq!(valor["alterado"], false);
        assert_eq!(valor["aberto_em"], serde_json::json!(["ORIGEM"]));
    }
    let unico = json(&saidas[0]);
    assert_eq!(unico["destino"]["id_unidade"], "2");
    assert!(unico.get("destinos").is_none());
    let plural = json(&saidas[1]);
    let ids: Vec<_> = plural["destinos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id_unidade"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["2", "3"]);
    assert_eq!(
        plural["depois_esperado"],
        serde_json::json!(["ORIGEM", "DESTINO", "ARQUIVO"])
    );
    assert!(pedidos.iter().any(|p| p == "GET grupo_unidade_consultar"));
    assert!(
        pedidos
            .iter()
            .any(|p| p == "POST unidade_auto_completar_envio_processo")
    );
    sem_mutacao(&pedidos, &depois);
}

#[test]
fn grupo_recusa_id_lexical_divergente_e_grupo_ausente_sem_escrever() {
    let divergente = ["--json", "processo", "enviar", NUP, "--grupo", "004"];
    let ausente = ["--json", "processo", "enviar", NUP, "--grupo", "4"];
    let (saidas, pedidos, depois) = executar_com_ensaio_local(
        &[&divergente, &ausente],
        false,
        Some("Nota da origem"),
        false,
        EnsaioRemocao::GrupoIdDivergente,
    );
    falha(&saidas[0], "IdentidadeDoMembroDivergente");
    falha(&saidas[1], "GrupoNaoEncontradoNaPagina");
    assert!(
        pedidos
            .iter()
            .any(|p| p == "POST unidade_auto_completar_envio_processo")
    );
    sem_mutacao(&pedidos, &depois);
}
