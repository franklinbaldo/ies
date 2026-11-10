//! Aceite da ciência na `UnB` 4.0.9 ao vivo (10/11/2026), sem `infra_hash`:
//! script em `<!--//--><![CDATA[//><!--` e, em `inicializar()`, uma chamada
//! simples antes da recarga; a segunda recarga, depois do `return`, não roda.
use super::{AlvoDaCiencia as Alvo, ErroCiencia as E, atividade_do_aceite};

fn aceite(antes: &str) -> String {
    format!(
        r#"<html><head><script type="text/javascript" charset="iso-8859-1" >
<!--//--><![CDATA[//><!--
var bolRedimensionando = false;
function inicializar(){{

  {antes}

  //exclusão/renúncia volta para o controle de processos
      atualizarArvore('controlador.php?acao=procedimento_visualizar&acao_origem=arvore_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=44935#ID-44935');
    return;
       atualizarArvore('controlador.php?acao=procedimento_visualizar&acao_origem=arvore_visualizar&id_procedimento=10&id_documento=&id_procedimento_anexado=');
     return;
}}
//--><!]]>
</script></head><body onload="inicializar();"></body></html>"#
    )
}

fn processo() -> Alvo<'static> {
    Alvo::Processo {
        id_procedimento: "10",
    }
}

#[test]
fn aceite_em_cdata_com_chamada_simples_antes_da_recarga() {
    assert_eq!(
        atividade_do_aceite(&aceite("exibirLinkVoltarArvoreVisualizar();"), processo()),
        Ok("44935".into())
    );
}

#[test]
fn instrucao_com_argumento_ou_condicao_antes_da_recarga_nao_prova() {
    for antes in ["if (x) return;", "return;", "alerta('x');", "x = 1;"] {
        assert_eq!(
            atividade_do_aceite(&aceite(antes), processo()),
            Err(E::AceiteSemAtividade),
            "{antes}"
        );
    }
}

#[test]
fn lista_com_contagem_na_legenda_comprova_atividade() {
    // UnB 4.0.9 ao vivo (10/11/2026): "Lista de Ciências (1 registro):".
    let html = "<html><table id='tblAndamentos' summary='Tabela de Ciências'><caption class='infraCaption'>Lista de Ciências (1 registro):</caption><tr><td><input class='infraCheckbox' id='chkInfraItem' name='chkInfraItem' type='checkbox' value='44936' /></td></tr></table></html>";
    assert_eq!(super::atividade_na_lista(html, "44936"), Ok(true));
    let outra = html.replace("(1 registro)", "(1 andamento)");
    assert!(super::atividade_na_lista(&outra, "44936").is_err());
}
