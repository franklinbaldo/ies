//! Replay offline da criação de processos (#245). Os corpos são inspecionados
//! exclusivamente por leitores usados pela CLI; POST gravado não prova efeito.
//! Os ZIPs são verificados antes da leitura e nunca contêm sessão em relatórios.
use crate::cassette::acervo::Resultado;
use serde_json::Value;

#[derive(Debug)]
struct Registro {
    leitor: &'static str,
    instalacao: String,
    passo: usize,
    resultado: Resultado,
}

// Shim RED: as leituras só passam a contar após a implementação dos leitores.
fn replay(_instalacao: &str, _cassette: &Value) -> Vec<Registro> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use serde_json::json;

    fn passo(operacao: &str, metodo: &str, status: u64, mime: &str, corpo: &str) -> Value {
        json!({
            "request": {"operation": operacao, "method": metodo, "pathname": "/sei/controlador.php"},
            "response": {
                "status_code": status,
                "url": format!("https://sei.treinamento.unb.br/sei/controlador.php?acao={operacao}"),
                "encoding": "utf-8",
                "headers": {"content-type": mime},
                "body_base64": STANDARD.encode(corpo.as_bytes())
            }
        })
    }

    fn lista(links: &str) -> String {
        format!(
            "<button onclick=\"exibirTiposProcedimento('U')\">Tudo</button><table id='tblTipoProcedimento'><tr><td>{links}</td></tr></table>"
        )
    }

    #[test]
    fn replay_reusa_leitores_de_lista_cadastro_e_hipotese() {
        // Cenário estrutural de Todos tests/test_processo_criar_escolher_tipo_010.py @323eb01.
        let tipo = "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12'>Ensaio</a>";
        let cadastro = "<form id='frmProcedimentoCadastro'><input name='hdnIdTipoProcedimento' value='12'><input name='txtDescricao'><input name='rdoNivelAcesso' value='1' type='radio'><select name='selAssuntos'><option value='21'>Assunto</option></select><select name='selHipoteseLegal'><option value='31'>Base legal</option></select></form>";
        let json = json!({"interactions": [
            passo("procedimento_escolher_tipo", "GET", 200, "text/html", &lista(tipo)),
            passo("procedimento_gerar", "GET", 200, "text/html", cadastro),
            passo("hipotese_legal_select_nome_base_legal", "POST", 200, "application/xml", "<select><option value='null'> </option><option value='31'>Base legal</option></select>")
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 3);
        assert_eq!(registros.iter().map(|r| &r.resultado).collect::<Vec<_>>(), vec![&Resultado::Lido; 3]);
    }

    #[test]
    fn replay_nao_confunde_post_com_prova_de_criacao() {
        let json = json!({"interactions": [
            passo("procedimento_gerar", "POST", 302, "text/html", "")
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
        assert_eq!(registros[0].resultado, Resultado::NaoAplicavel("post_sem_leitura_verificadora"));
    }

    #[test]
    fn replay_detecta_nome_de_tipo_duplicado_pelo_leitor_real() {
        let tipo = "<a class='ancoraOpcao' href='controlador.php?acao=procedimento_gerar&amp;id_tipo_procedimento=12'>Ensaio</a>";
        let json = json!({"interactions": [
            passo("procedimento_escolher_tipo", "GET", 200, "text/html", &lista(&format!("{tipo}{tipo}")))
        ]});
        let registros = replay("unb", &json);
        assert_eq!(registros.len(), 1);
        assert_eq!(registros[0].resultado, Resultado::Recusado("tipo_de_processo_ambiguo"));
    }
}
