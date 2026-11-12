use super::*;
use crate::formulario_web::CampoDeTexto;
use crate::tela_html::HtmlServido;

const ID: &str = "12345";
fn cadastro(grupos: &str) -> String {
    format!(
        r#"<form id="frmAcompanhamentoCadastro" method="post"
    action="controlador.php?acao=acompanhamento_cadastrar&amp;id_procedimento=12345&amp;infra_hash=SYNTHETIC">
    <input type="hidden" name="hdnIdProtocolo" value="12345">
    <select name="selGrupoAcompanhamento">{grupos}</select>
    <textarea name="txaObservacao"></textarea>
    <button type="submit" name="sbmCadastrarAcompanhamento">Salvar</button></form>"#
    )
}
fn tabela(linhas: &str) -> String {
    format!(
        r#"<table summary="Tabela de Acompanhamentos.">
    <tr><th></th><th>Grupo</th><th>Observação</th><th>Usuário</th><th>Data</th><th>Ações</th></tr>{linhas}</table>"#
    )
}
fn linha(id: &str, grupo: &str, obs: &str) -> String {
    format!(
        r#"<tr><td><input type="checkbox" value="{id}" /></td>
    <td>{grupo}</td><td>{obs}</td><td>aluno</td><td>15/10/2026</td><td>ação</td></tr>"#
    )
}
fn a(id: &str, grupo: &str, obs: &str) -> Acompanhamento {
    Acompanhamento {
        id: id.into(),
        grupo: grupo.into(),
        observacao: obs.into(),
    }
}
#[test]
fn grupos_da_tela_observada_e_nome_exato() {
    // Caso de todos tests/test_processo_acompanhar_010.py @323eb01.
    let html = HtmlServido::de_teste(&cadastro(
        r#"<option value="null">&nbsp;</option>
       <option value="19">acomp fiel</option><option value="64">Boas-Vindas</option>"#,
    ));
    let grupos = grupos_do_cadastro(&html, ID).unwrap();
    assert_eq!(
        escolher_grupo(&grupos, Some("acomp fiel")).unwrap().valor,
        "19"
    );
    assert_eq!(escolher_grupo(&grupos, None).unwrap().valor, "null");
    assert_eq!(
        escolher_grupo(&grupos, Some("ACOMP FIEL")),
        Err(ErroAcompanhar::GrupoNaoOferecido)
    );
}
#[test]
fn recusa_cadastro_de_outro_processo_e_acao_duplicada() {
    let html = cadastro(r#"<option value="19">G</option>"#);
    assert_eq!(
        grupos_do_cadastro(&HtmlServido::de_teste(&html), "999"),
        Err(ErroAcompanhar::CadastroDeOutroProcesso)
    );
    let duplicado = HtmlServido::de_teste(&html.replace(
        "acao=acompanhamento_cadastrar",
        "acao=acompanhamento_cadastrar&amp;acao=acompanhamento_cadastrar",
    ));
    assert_eq!(
        grupos_do_cadastro(&duplicado, ID),
        Err(ErroAcompanhar::ChaveDoCadastroIlegivel(
            ErroChaveDaConsulta::Repetida { chave: "acao" }
        ))
    );
}

#[test]
fn action_com_par_sem_igual_recusa_com_a_causa() {
    let html = cadastro(r#"<option value="19">G</option>"#);
    let com_lixo = HtmlServido::de_teste(&html.replace(
        "acao=acompanhamento_cadastrar&amp;id_procedimento",
        "acao=acompanhamento_cadastrar&amp;lixo&amp;id_procedimento",
    ));
    assert_eq!(
        grupos_do_cadastro(&com_lixo, ID),
        Err(ErroAcompanhar::ConsultaDoCadastroIlegivel(
            ErroConsultaDoControlador::ParSemIgual
        ))
    );
    let id_repetido = HtmlServido::de_teste(&html.replace(
        "id_procedimento=12345",
        "id_procedimento=12345&amp;id_procedimento=12345",
    ));
    assert_eq!(
        grupos_do_cadastro(&id_repetido, ID),
        Err(ErroAcompanhar::ChaveDoCadastroIlegivel(
            ErroChaveDaConsulta::Repetida {
                chave: "id_procedimento"
            }
        ))
    );
}
#[test]
fn grupos_ambiguos_e_ausentes_recusados() {
    let html = HtmlServido::de_teste(&cadastro(
        r#"<option value="1">Mesmo</option><option value="2">Mesmo</option>"#,
    ));
    let grupos = grupos_do_cadastro(&html, ID).unwrap();
    assert_eq!(
        escolher_grupo(&grupos, Some("Mesmo")),
        Err(ErroAcompanhar::GrupoAmbiguo)
    );
    assert_eq!(
        escolher_grupo(&grupos, None),
        Err(ErroAcompanhar::GrupoNaoOferecido)
    );
}
#[test]
fn lista_sem_tabela_nao_prova_vazia() {
    // Caso de todos tests/test_processo_acompanhar_010.py @323eb01.
    assert_eq!(
        acompanhamentos_da_lista(&HtmlServido::de_teste("<html>cortada</html>")),
        Err(ErroAcompanhar::ListaNaoPublicada)
    );
    assert_eq!(
        acompanhamentos_da_lista(&HtmlServido::de_teste(&tabela(""))),
        Ok(vec![])
    );
}
#[test]
fn lista_identidades_duplicadas_falha_fechado() {
    let html = HtmlServido::de_teste(&tabela(&format!(
        "{}{}",
        linha("831", "Grupo", "O1"),
        linha("831", "G2", "O2")
    )));
    assert_eq!(
        acompanhamentos_da_lista(&html),
        Err(ErroAcompanhar::IdDeAcompanhamentoRepetido)
    );
}
#[test]
fn evita_duplicata_antes_de_enviar() {
    assert_eq!(
        conferir_novo_grupo(&[a("1", "G", "Antiga")], "G"),
        Err(ErroAcompanhar::GrupoJaAcompanhado)
    );
    assert!(conferir_novo_grupo(&[a("1", "G", "Antiga")], "Outra").is_ok());
}
#[test]
fn remocao_exige_alvo_sem_ambiguidade() {
    let antes = vec![a("1", "G", "A"), a("2", "H", "B")];
    assert_eq!(
        alvo_para_deixar(&antes, None),
        Err(ErroAcompanhar::EscolhaDeAcompanhamentoAmbigua)
    );
    assert_eq!(alvo_para_deixar(&antes, Some("H")).unwrap().id, "2");
}
#[test]
fn reconciliacao_so_aceita_uma_alteracao_precisa() {
    let antes = vec![a("1", "G", "A")];
    let depois = vec![a("1", "G", "A"), a("2", "H", "B")];
    assert!(conferir_inclusao(&antes, &depois, "H", "B").is_ok());
    assert_eq!(
        conferir_inclusao(&antes, &[a("1", "G", "A")], "H", "B"),
        Err(ErroAcompanhar::AcompanhamentoNaoConfirmado)
    );
    assert_eq!(
        conferir_inclusao(&antes, &[a("1", "G", "MUDOU"), a("2", "H", "B")], "H", "B"),
        Err(ErroAcompanhar::AcompanhamentoNaoConfirmado)
    );
    assert!(conferir_remocao(&depois, &antes, "2").is_ok());
    assert_eq!(
        conferir_remocao(&depois, &antes, "1"),
        Err(ErroAcompanhar::AcompanhamentoNaoConfirmado)
    );
}
#[test]
fn lista_realista_com_entidades() {
    let html = HtmlServido::de_teste(&tabela(&linha(
        "831",
        "Boas-Vindas",
        "Prazos &amp; revisões",
    )));
    let lida = acompanhamentos_da_lista(&html).unwrap();
    assert_eq!(lida[0], a("831", "Boas-Vindas", "Prazos & revisões"));
}

// Casos do Todos tests/test_processo_acompanhar_010.py @323eb01:
// submissão de cadastro derivada da tela servida, nunca de endpoint inventado.
#[test]
fn submissao_inclusao_reflete_grupo_exato_oculto_e_botao() {
    use crate::formulario_web::{CodificacaoDaTela, TelaServida};
    use crate::instalacao::Instalacao;
    let html =
        cadastro(r#"<option value="null">&nbsp;</option><option value="19">acomp fiel</option>"#)
            .replace(
                "infra_hash=SYNTHETIC",
                &format!("infra_hash={}", "a".repeat(64)),
            )
            .replace(
                r#"name="sbmCadastrarAcompanhamento">"#,
                r#"name="sbmCadastrarAcompanhamento" value="Salvar">"#,
            );
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    );
    let tela = TelaServida {
        url: &url,
        html: &HtmlServido::de_teste(&html),
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let sub = super::submissao::derivar_inclusao(
        &instalacao,
        &tela,
        ID,
        "TESTE",
        Some("acomp fiel"),
        CampoDeTexto::Preenchido("Prazo."),
    )
    .expect("formulário sintético válido");
    assert_eq!(
        sub.nomes_dos_campos(),
        &[
            "hdnIdProtocolo",
            "selGrupoAcompanhamento",
            "txaObservacao",
            "sbmCadastrarAcompanhamento",
        ]
    );
    assert_eq!(
        sub.corpo(),
        b"hdnIdProtocolo=12345&selGrupoAcompanhamento=19&txaObservacao=Prazo.&sbmCadastrarAcompanhamento=Salvar"
    );
    assert_eq!(sub.referer(), url);
    assert_eq!(sub.origem(), "https://sei.exemplo.gov.br");
    assert!(!format!("{sub:?}").contains(&"a".repeat(64)));
}

/// Sem `--grupo` e sem `--observacao`: a opção [`SEM_GRUPO`] publicada pelo
/// SEI (`value="null"`, `&nbsp;`) e `txaObservacao` presente e vazio, como o
/// navegador envia o `textarea` vazio do cadastro.
#[test]
fn submissao_inclusao_sem_grupo_nem_observacao_envia_os_campos_vazios() {
    use crate::formulario_web::{CodificacaoDaTela, TelaServida};
    use crate::instalacao::Instalacao;
    let html =
        cadastro(r#"<option value="null">&nbsp;</option><option value="19">acomp fiel</option>"#)
            .replace(
                "infra_hash=SYNTHETIC",
                &format!("infra_hash={}", "a".repeat(64)),
            )
            .replace(
                r#"name="sbmCadastrarAcompanhamento">"#,
                r#"name="sbmCadastrarAcompanhamento" value="Salvar">"#,
            );
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    );
    let tela = TelaServida {
        url: &url,
        html: &HtmlServido::de_teste(&html),
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let sub = super::submissao::derivar_inclusao(
        &instalacao,
        &tela,
        ID,
        "TESTE",
        None,
        CampoDeTexto::de(None),
    )
    .expect("formulário sintético válido");
    assert_eq!(
        sub.corpo(),
        b"hdnIdProtocolo=12345&selGrupoAcompanhamento=null&txaObservacao=&sbmCadastrarAcompanhamento=Salvar"
    );
}

#[test]
fn submissao_inclusao_nao_inventa_grupo_ou_endereco() {
    use crate::formulario_web::{CodificacaoDaTela, TelaServida};
    use crate::instalacao::Instalacao;
    let html = cadastro(r#"<option value="19">acomp fiel</option>"#).replace(
        "infra_hash=SYNTHETIC",
        &format!("infra_hash={}", "a".repeat(64)),
    );
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    );
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let tela = TelaServida {
        url: &url,
        html: &HtmlServido::de_teste(&html),
        codificacao: CodificacaoDaTela::Iso88591,
    };
    assert!(matches!(
        super::submissao::derivar_inclusao(
            &instalacao,
            &tela,
            ID,
            "TESTE",
            Some("inexistente"),
            CampoDeTexto::CampoVazio
        ),
        Err(ErroAcompanhar::GrupoNaoOferecido)
    ));
    let externo = url.replace("sei.exemplo.gov.br", "outro.exemplo.gov.br");
    assert!(
        super::submissao::derivar_inclusao(
            &instalacao,
            &TelaServida {
                url: &externo,
                ..tela
            },
            ID,
            "TESTE",
            Some("acomp fiel"),
            CampoDeTexto::CampoVazio,
        )
        .is_err()
    );
}

#[test]
fn submissao_inclusao_recusa_hash_duplicado_e_processo_divergente() {
    use crate::formulario_web::{CodificacaoDaTela, TelaServida};
    use crate::instalacao::Instalacao;
    let html = cadastro(r#"<option value="19">acomp fiel</option>"#).replace(
        "infra_hash=SYNTHETIC",
        &format!("infra_hash={}", "a".repeat(64)),
    );
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=acompanhamento_cadastrar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    );
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let recusada = |html: &str| {
        super::submissao::derivar_inclusao(
            &instalacao,
            &TelaServida {
                url: &url,
                html: &HtmlServido::de_teste(html),
                codificacao: CodificacaoDaTela::Iso88591,
            },
            ID,
            "TESTE",
            Some("acomp fiel"),
            CampoDeTexto::CampoVazio,
        )
        .is_err()
    };
    assert!(recusada(
        &html.replace("id_procedimento=12345", "id_procedimento=99999")
    ));
    assert!(recusada(&html.replace(
        "infra_hash=",
        "infra_hash=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc&infra_hash="
    )));
    assert!(recusada(&html.replace(
        "name=\"hdnIdProtocolo\" value=\"12345\"",
        "name=\"hdnIdProtocolo\" value=\"99999\""
    )));
}
