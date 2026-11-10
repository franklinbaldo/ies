//! Tela do nó da `UnB` 4.0.9 ao vivo (10/11/2026), sem `infra_hash`: script
//! dentro de `<!--//--><![CDATA[//><!--`, `confirm` comentado e a função
//! vizinha `cienciaProcessoAnexado`, que não é a ciência do processo. A
//! retirada do sobrestamento usa o `confirm` ativo, sem `//`.
use super::super::sobrestar::ler_endereco_de_retomar;
use super::{url_da_ciencia, url_da_ciencia_do_documento};
use crate::instalacao::Instalacao;

const TELA: &str = r#"<html><head>
<script type="text/javascript" charset="iso-8859-1" >
<!--//--><![CDATA[//><!--
function cienciaProcesso(){
  //if (confirm('Confirma ciência no processo?')){
   location.href = 'controlador.php?acao=procedimento_ciencia&acao_origem=arvore_visualizar&id_procedimento=51897&atualizar_arvore=1&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash=9999999999999999999999999999999999999999999999999999999999999999';
  //}
}

function cienciaDocumento(){
  //if (confirm('Confirma ciência no documento?')){
   location.href = 'controlador.php?acao=documento_ciencia&acao_origem=arvore_visualizar&id_procedimento=51897&id_documento=51898&atualizar_arvore=1&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash=9999999999999999999999999999999999999999999999999999999999999999';
  //}
}

function cienciaProcessoAnexado(){
  //if (confirm('Confirma ciência no processo anexado?')){
  location.href = 'controlador.php?acao=procedimento_anexado_ciencia&acao_origem=arvore_visualizar&id_procedimento=51897&id_procedimento_anexado=&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash=9999999999999999999999999999999999999999999999999999999999999999';
  //}
}
function removerSobrestamentoProcesso(){
  if (confirm('Confirma remoção de sobrestamento do processo?')){
   location.href = 'controlador.php?acao=procedimento_remover_sobrestamento&acao_origem=arvore_visualizar&id_procedimento=51897&atualizar_arvore=1&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash=9999999999999999999999999999999999999999999999999999999999999999';
  }
}
//--><!]]>
</script></head><body></body></html>"#;

fn unb() -> Instalacao {
    Instalacao::interpretar("https://sei.treinamento.unb.br/sei").unwrap()
}

#[test]
fn ciencia_do_processo_ignora_a_funcao_do_processo_anexado() {
    let r = url_da_ciencia(&unb(), TELA, "51897");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn ciencia_do_documento_le_script_em_cdata() {
    let r = url_da_ciencia_do_documento(&unb(), TELA, "51898", "51897");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn retomada_aceita_confirm_ativo() {
    let r = ler_endereco_de_retomar(&unb(), TELA, "51897");
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn ciencia_do_processo_pela_variavel_do_sei_41_e_50() {
    // UFRJ e GO ao vivo (10/11/2026): a URL vem em variáveis, com a do
    // processo anexado logo abaixo.
    let h = "&infra_sistema=100000100&infra_unidade_atual=9516&infra_hash=9999999999999999999999999999999999999999999999999999999999999999";
    let tela = format!(
        "<html><head><script>\nvar linkCienciaProcesso = 'controlador.php?acao=procedimento_ciencia&acao_origem=arvore_visualizar&id_procedimento=51897&atualizar_arvore=1{h}';\nvar linkCienciaProcessoAnexado = 'controlador.php?acao=procedimento_anexado_ciencia&id_procedimento=51897&id_procedimento_anexado={h}';\n</script></head></html>"
    );
    let r = url_da_ciencia(&unb(), &tela, "51897");
    assert!(r.is_ok(), "{r:?}");
}
