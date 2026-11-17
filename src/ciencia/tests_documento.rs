//! RED/GREEN da ciência documental: Todos `test_documento_ciente_010.py`
//! @323eb01, HTML sintético (sem segredos) e contagem exata de GETs.
use super::documento::{ErroCienciaDocumento, PedidoDaCienciaDocumento, ciencia_no_documento};
use crate::arvore_processo::tests_assinatura::{
    arvore_assinavel, barra_assinavel, link_da_tela_do_documento, tela_do_documento_40,
    tela_do_documento_50,
};
use crate::documento::NumeroDocumento;
use crate::processo_concluir::tests_apoio::*;
use crate::tela_html::HtmlServido;
use crate::transporte::FalhaTransporte;

const DOC: &str = "0000101";
const ID_DOC: &str = "21";
fn url_da_tela_do_documento() -> String {
    absoluta(&link_da_tela_do_documento(ID, ID_DOC))
}

const ATIVIDADE: &str = "42457";
const BOTAO: &str = "<a href=\"#\" onclick=\"cienciaDocumento();\" title=\"Ciência\">Ciência</a>";

fn arvore_com_botao() -> String {
    let barra = format!("{}{}", barra_assinavel(ID, ID_DOC, Some('S')), BOTAO);
    arvore_assinavel(ID, ID_DOC, DOC, &barra, &[])
}

fn arvore_com_lista() -> String {
    let mut html = arvore_com_botao();
    let acao = format!(
        "NosAcoes[9] = new infraArvoreAcao(\"CIENCIAS\",\"CD{ID_DOC}\",\"{ID_DOC}\",\"{link}\",\"ifrVisualizacao\",\"Visualizar Ciências\",\"svg/ciencia.svg\",true);\n",
        link = link_da_lista()
    );
    html = html.replacen(
        "  if (Nos.length){",
        &format!("{acao}  if (Nos.length){{"),
        1,
    );
    html
}

fn link_da_lista() -> String {
    format!(
        "controlador.php?acao=protocolo_ciencia_listar&id_procedimento={ID}&id_documento={ID_DOC}&infra_hash={}",
        "6".repeat(64)
    )
}

fn link_da_escrita() -> String {
    format!(
        "controlador.php?acao=documento_ciencia&id_documento={ID_DOC}&infra_hash={}",
        "7".repeat(64)
    )
}

fn aceite(atividade: &str) -> String {
    format!(
        "<html><head><script>function inicializar(){{ atualizarArvore(         'controlador.php?acao=procedimento_visualizar&id_procedimento={ID}&id_documento={ID_DOC}&documento_visualizar_ciencias=1&id_atividade={atividade}#ID-{atividade}'); }}</script></head></html>"
    )
}

fn lista(atividade: &str) -> String {
    format!(
        "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption>         <tr><td><input type='checkbox' name='chkInfraItem' value='{atividade}' /></td></tr>         </table></html>"
    )
}

fn rodar(
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (
    Result<super::documento::CienciaDoDocumento, ErroCienciaDocumento>,
    Vec<Pedido>,
) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let doc = NumeroDocumento::novo(DOC).unwrap();
        ciencia_no_documento(
            sessao,
            &PedidoDaCienciaDocumento {
                numero: &doc,
                processo: Some(nup),
                unidade: None,
                confirmar,
            },
            indice,
            rede,
        )
    })
}

#[test]
fn previa_tres_get_sem_escrita() {
    let (resultado, pedidos) = rodar(false, abertura(arvore_com_botao()));
    let feito = resultado.unwrap();
    assert!(!feito.confirmado && !feito.verificado);
    assert_eq!(feito.requisicoes, 3);
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn barra_sem_ciencia_recusa_antes_de_get_mutador() {
    let arvore = arvore_assinavel(
        ID,
        ID_DOC,
        DOC,
        &barra_assinavel(ID, ID_DOC, Some('S')),
        &[],
    );
    let (resultado, pedidos) = rodar(true, abertura(arvore));
    assert!(matches!(
        resultado,
        Err(ErroCienciaDocumento::CienciaNaoOferecida)
    ));
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn confirmar_sete_gets_exatos_com_um_mutador_e_lista_do_documento() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(aceite(ATIVIDADE)),
        ok(arvore_com_lista()),
        ok(lista(ATIVIDADE)),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let feito = resultado.unwrap();
    assert!(feito.confirmado && feito.verificado);
    assert_eq!(feito.id_atividade.as_deref(), Some(ATIVIDADE));
    assert_eq!(feito.requisicoes, 7);
    let mut esperado = pedidos_da_abertura();
    esperado.extend([
        get(&url_da_tela_do_documento()),
        get(&absoluta(&link_da_escrita())),
        get(&url_arvore()),
        get(&absoluta(&link_da_lista())),
    ]);
    assert_eq!(pedidos, esperado);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
}

#[test]
fn aceite_de_outro_documento_nunca_prova_ciencia() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(aceite(ATIVIDADE).replace(&format!("id_documento={ID_DOC}"), "id_documento=99")),
        ok(arvore_com_lista()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(matches!(
        resultado,
        Err(ErroCienciaDocumento::NaoConfirmado { .. })
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
}

#[test]
fn falha_da_escrita_nao_repetida_seguida_de_leitura() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        Err(FalhaTransporte::Rede),
        ok(arvore_com_lista()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(
        erro,
        ErroCienciaDocumento::NaoConfirmado {
            falha_do_aceite: Some(super::documento::FalhaDoAceite::Transporte(
                FalhaTransporte::Rede
            )),
            requisicoes: 6,
            ..
        }
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

// RED #271: recusa HTTP no GET mutador não pode perder status e motivo.
#[test]
fn http_403_apos_get_mutador_preserva_status_e_rele_controle() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        Ok((
            403,
            "<html><body>Solicitacao recusada</body></html>".to_owned(),
            None,
        )),
        ok(arvore_com_lista()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(
        erro,
        ErroCienciaDocumento::NaoConfirmado {
            falha_do_aceite: Some(super::documento::FalhaDoAceite::Transporte(
                FalhaTransporte::Status(crate::transporte::StatusHttpRecusado {
                    status: 401 | 403,
                    ..
                })
            )),
            requisicoes: 6,
            ..
        }
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

// RED #271: validação Infra em resposta 200 é recusa, não aceite.
#[test]
fn validacao_da_infra_no_get_mutador_preserva_mensagem_e_rele() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(
            "<html><textarea id='txaInfraValidacao'>Solicitacao recusada</textarea></html>"
                .to_owned(),
        ),
        ok(arvore_com_lista()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    let ErroCienciaDocumento::NaoConfirmado {
        falha_do_aceite: Some(super::documento::FalhaDoAceite::ValidacaoDaInfra(recusa)),
        ..
    } = &erro
    else {
        panic!("{erro:?}");
    };
    assert_eq!(Some(recusa.as_str()), Some("Solicitacao recusada"));
    assert!(matches!(
        erro,
        ErroCienciaDocumento::NaoConfirmado {
            falha_do_aceite: Some(super::documento::FalhaDoAceite::ValidacaoDaInfra(_)),
            requisicoes: 6,
            ..
        }
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn lista_com_atividade_divergente_nao_prova() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_50(ID, ID_DOC)),
        ok(aceite(ATIVIDADE)),
        ok(arvore_com_lista()),
        ok(lista("42458")),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(matches!(
        resultado,
        Err(ErroCienciaDocumento::NaoConfirmado { .. })
    ));
    assert_eq!(pedidos.len(), 7);
}

#[test]
fn fixture_da_lista_publica_link_do_documento_conferido() {
    let html = arvore_com_lista();
    assert!(html.contains("NosAcoes[9]"));
    let inst = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    let lista = crate::arvore_processo::lista_do_documento(&inst, &html, ID, ID_DOC);
    assert!(matches!(lista, Ok(Some(_))), "leitura de lista: {lista:?}");
}

#[test]
fn fixture_do_aceite_publica_atividade_do_documento_correto() {
    let atividade = super::atividade_do_aceite(
        &HtmlServido::de_teste(&aceite(ATIVIDADE)),
        super::AlvoDaCiencia::Documento {
            id_procedimento: ID,
            id_documento: ID_DOC,
        },
    );
    assert_eq!(atividade, Ok(ATIVIDADE.to_owned()));
}

#[test]
fn template_inerte_nao_autoriza_get_mutador() {
    // Oráculo: linkCienciaDocumento só é acionável no script ativo da tela.
    // Um template inerte não pode autorizar a escrita do usuário da sessão.
    let tela = format!(
        "<html><body><template>\nvar linkCienciaDocumento = '{}';\n</template></body></html>",
        link_da_escrita()
    );
    let mut respostas = abertura(arvore_com_botao());
    respostas.push(ok(tela));
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(
        matches!(resultado, Err(ErroCienciaDocumento::Ciencia(_))),
        "{resultado:?}"
    );
    assert_eq!(pedidos.len(), 4);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        0
    );
}

fn rejeitar_tela_sem_get_mutador(tela: String) {
    let mut respostas = abertura(arvore_com_botao());
    respostas.push(ok(tela));
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(
        matches!(resultado, Err(ErroCienciaDocumento::Ciencia(_))),
        "{resultado:?}"
    );
    assert_eq!(pedidos.len(), 4, "a prévia e a tela são somente leitura");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        0,
        "nunca enviar GET mutador a partir de JS inerte"
    );
}

// RED #271: script aninhado em contêiner inerte não é ação ofertada.
#[test]
fn script_dentro_de_template_nao_autoriza_get_mutador() {
    let tela = format!(
        "<html><template><script>var linkCienciaDocumento = '{}';</script></template></html>",
        link_da_escrita()
    );
    rejeitar_tela_sem_get_mutador(tela);
}

#[test]
fn comentario_de_bloco_nao_autoriza_get_mutador() {
    let tela = format!(
        "<html><script>/*\nvar linkCienciaDocumento = '{}';\n*/</script></html>",
        link_da_escrita()
    );
    rejeitar_tela_sem_get_mutador(tela);
}

#[test]
fn ramo_if_false_nao_autoriza_get_mutador() {
    let tela = format!(
        "<html><script>if (false) {{\nvar linkCienciaDocumento = '{}';\n}}</script></html>",
        link_da_escrita()
    );
    rejeitar_tela_sem_get_mutador(tela);
}

#[test]
fn if_false_sem_chaves_nao_autoriza_get_mutador() {
    let tela = format!(
        "<html><script>if (false)\nvar linkCienciaDocumento = '{}';</script></html>",
        link_da_escrita()
    );
    rejeitar_tela_sem_get_mutador(tela);
}

#[test]
fn funcao_somente_comentada_nao_autoriza_get_mutador() {
    let funcao = tela_do_documento_40(ID, ID_DOC);
    let trecho = funcao
        .split("<script type=\"text/javascript\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let tela = format!("<html><script>/*\n{trecho}\n*/</script></html>");
    rejeitar_tela_sem_get_mutador(tela);
}

#[test]
fn funcao_em_ramo_inativo_nao_autoriza_get_mutador() {
    let funcao = tela_do_documento_40(ID, ID_DOC);
    let trecho = funcao
        .split("<script type=\"text/javascript\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let tela = format!("<html><script>if (false) {{\n{trecho}\n}}</script></html>");
    rejeitar_tela_sem_get_mutador(tela);
}

// RED/GREEN #271: uma atividade em tabela inteiramente inerte não é prova
// de que o GET mutador registrou ciência, mesmo com id_atividade no aceite.
#[test]
fn tabela_da_lista_em_template_nao_confirma_apos_get_mutador() {
    let html = lista(ATIVIDADE)
        .replacen(
            "<table id='tblAndamentos'",
            "<template><table id='tblAndamentos'",
            1,
        )
        .replacen("</table>", "</table></template>", 1);
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(aceite(ATIVIDADE)),
        ok(arvore_com_lista()),
        ok(html),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(
        erro,
        ErroCienciaDocumento::NaoConfirmado { requisicoes: 7, .. }
    ));
    assert_eq!(pedidos.len(), 7);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1,
        "não repetir o GET mutador"
    );
    assert_eq!(pedidos.last(), Some(&get(&absoluta(&link_da_lista()))));
}

#[path = "tests_documento/efeitos_incertos.rs"]
mod efeitos_incertos;

// RED #271: a recusa Infra do documento tem status real HTTP 200.
// JSON deve preservá-lo para que o usuário entenda a causa do efeito incerto.
#[test]
fn json_documento_recusado_infra_200_exibe_status_real_e_contagem() {
    let mut respostas = abertura(arvore_com_botao());
    respostas.extend([
        ok(tela_do_documento_40(ID, ID_DOC)),
        ok(
            "<html><textarea id='txaInfraValidacao'>Solicitacao recusada</textarea></html>"
                .to_owned(),
        ),
        ok(arvore_com_lista()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.expect_err("validacao Infra nao e sucesso");
    let ErroCienciaDocumento::NaoConfirmado {
        falha_do_aceite: Some(super::documento::FalhaDoAceite::ValidacaoDaInfra(recusa)),
        requisicoes: 6,
        ..
    } = &erro
    else {
        panic!("{erro:?}");
    };
    assert_eq!(Some(recusa.as_str()), Some("Solicitacao recusada"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == absoluta(&link_da_escrita()))
            .count(),
        1
    );
}
