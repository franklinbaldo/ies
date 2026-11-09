//! Projeção GET do formulário de atribuição observado no SQLite efêmero.
//! Escrita fechada: nenhuma resposta sintética confirma um POST de atribuição.
use crate::{Estado, Resposta, parametro_unico};
use iesei::atribuicoes;
use iesei::banco_cassettes;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

#[must_use]
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let unidade = match banco_cassettes::ler(banco, instalacao, cenario) {
        Ok(Some(controle)) => controle.unidade.as_str().to_owned(),
        Ok(None) | Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let Ok(Some(observacao)) = atribuicoes::ler(banco, instalacao, cenario, id, &unidade) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let mut html = format!(
        "<html><body><form id='frmAtividadeAtribuir' method='post' \
         action='controlador.php?acao=procedimento_atribuicao_cadastrar&amp;id_procedimento={id}&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
         <input type='hidden' name='hdnIdProtocolo' value='{id}'>\
         <select id='selAtribuicao' name='selAtribuicao'>\
         <option value='null' selected='selected'>Selecione</option>"
    );
    for usuario in observacao.usuarios {
        write!(
            html,
            "<option value='{}'>{} - {}</option>",
            crate::escapar(&usuario.id),
            crate::escapar(&usuario.login),
            crate::escapar(&usuario.nome)
        )
        .expect("formatação em String");
    }
    html.push_str(
        "</select><button type='submit' name='sbmSalvar' value='Salvar'>Salvar</button></form></body></html>",
    );
    Resposta::html(html)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rotas;
    use iesei::atribuicoes::{Observacao, Usuario};
    use iesei::banco_cassettes::{ControleResumido, Linha};
    use iesei::observacao::{TabelaObservada, UnidadeExibida};
    use iesei::processos::{self, ProcessoSimulado};

    const CENARIO: &str = "bdd-264-02-post";
    const URL: &str =
        "/sei/controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento=10";

    fn fixture() -> (Connection, Instalacao) {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        banco_cassettes::guardar(
            &mut banco,
            CENARIO,
            &ControleResumido {
                instalacao: origem.clone(),
                unidade: UnidadeExibida::nova("TESTE").unwrap(),
                recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
                gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            },
        )
        .unwrap();
        processos::guardar(
            &mut banco,
            &origem,
            CENARIO,
            &ProcessoSimulado {
                id: "10".into(),
                numero: "00001.000001/2026-01".into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
        atribuicoes::registrar(
            &mut banco,
            &origem,
            CENARIO,
            "10",
            "TESTE",
            &Observacao {
                login_atual: "aluno1".into(),
                usuarios: vec![
                    Usuario { id: "7".into(), login: "aluno1".into(), nome: "Pessoa 1".into() },
                    Usuario { id: "8".into(), login: "aluno2".into(), nome: "Pessoa 2".into() },
                ],
            },
        )
        .unwrap();
        (banco, origem)
    }

    fn leitura(banco: &Connection, origem: &Instalacao) -> String {
        atribuicoes::ler(banco, origem, CENARIO, "10", "TESTE")
            .unwrap()
            .unwrap()
            .login_atual
    }

    fn post(banco: &Connection, origem: &Instalacao, corpo: &str) -> Resposta {
        rotas::responder("POST", URL, corpo.as_bytes(), banco, origem, CENARIO)
    }

    const ATRIBUIR: &str =
        "hdnInfraTipoPagina=2&hdnIdProtocolo=10&selAtribuicao=8&sbmSalvar=Salvar";
    const REMOVER: &str =
        "hdnInfraTipoPagina=2&hdnIdProtocolo=10&selAtribuicao=null&sbmSalvar=Salvar";

    #[test]
    fn bdd_264_02_post_unico_atualiza_sqlite_e_arvore_independente() {
        let (banco, origem) = fixture();
        let antes = rotas::responder(
            "GET", URL, b"", &banco, &origem, CENARIO
        );
        assert_eq!(antes.estado, Estado::Ok);
        assert!(antes.corpo.contains("value='8'"));
        assert_eq!(leitura(&banco, &origem), "aluno1");
        let resposta = post(&banco, &origem, ATRIBUIR);
        assert_eq!(resposta.estado, Estado::Redirecionamento);
        assert_eq!(
            resposta.localizacao.as_deref(),
            Some("controlador.php?acao=arvore_visualizar&acao_origem=procedimento_atribuicao_cadastrar&id_procedimento=10"),
        );
        let arvore = rotas::responder(
            "GET",
            "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
            b"", &banco, &origem, CENARIO,
        );
        assert_eq!(arvore.estado, Estado::Ok);
        assert!(arvore.corpo.contains("atribuído para "), "{}", arvore.corpo);
        assert!(arvore.corpo.contains("aluno2"), "{}", arvore.corpo);
        assert_eq!(leitura(&banco, &origem), "aluno2");
        let repeticao = post(&banco, &origem, ATRIBUIR);
        assert_eq!(repeticao.estado, Estado::RequisicaoInvalida);
        assert_eq!(leitura(&banco, &origem), "aluno2");
    }

    #[test]
    fn bdd_264_03_remover_exige_atribuicao_existente_e_releitura() {
        let (banco, origem) = fixture();
        let resposta = post(&banco, &origem, REMOVER);
        assert_eq!(resposta.estado, Estado::Redirecionamento);
        assert_eq!(leitura(&banco, &origem), "");
        let arvore = rotas::responder(
            "GET",
            "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10",
            b"", &banco, &origem, CENARIO,
        );
        assert_eq!(arvore.estado, Estado::Ok);
        assert!(!arvore.corpo.contains("atribuído para "));
        assert_eq!(post(&banco, &origem, REMOVER).estado, Estado::RequisicaoInvalida);
    }

    #[test]
    fn rejeita_formulario_invalido_e_identidade_ou_opcao_nao_oferecidas() {
        let (banco, origem) = fixture();
        for corpo in [
            "hdnInfraTipoPagina=2&hdnIdProtocolo=11&selAtribuicao=8&sbmSalvar=Salvar",
            "hdnInfraTipoPagina=2&hdnIdProtocolo=10&selAtribuicao=99&sbmSalvar=Salvar",
            "hdnInfraTipoPagina=1&hdnIdProtocolo=10&selAtribuicao=8&sbmSalvar=Salvar",
            "hdnInfraTipoPagina=2&hdnIdProtocolo=10&selAtribuicao=8&selAtribuicao=7&sbmSalvar=Salvar",
            "hdnInfraTipoPagina=2&hdnIdProtocolo=10&selAtribuicao=8",
        ] {
            assert_eq!(
                post(&banco, &origem, corpo).estado,
                Estado::RequisicaoInvalida,
                "{corpo}"
            );
            assert_eq!(leitura(&banco, &origem), "aluno1");
        }
    }

    #[test]
    fn nao_inventa_atribuicao_se_ausente_observacao_ou_unidade() {
        let (banco, origem) = fixture();
        banco.execute("DELETE FROM processo_atribuicao_observada", []).unwrap();
        assert_eq!(post(&banco, &origem, ATRIBUIR).estado, Estado::DadosInsuficientes);
        let (banco, origem) = fixture();
        banco.execute("UPDATE controle_resumido SET unidade='OUTRA'", []).unwrap();
        assert_eq!(post(&banco, &origem, ATRIBUIR).estado, Estado::DadosInsuficientes);
        assert_eq!(leitura(&banco, &origem), "aluno1");
    }
}
