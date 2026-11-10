//! RED/GREEN da resposta do salvar do Nível de detalhe e dos campos exatos
//! do POST (contracts/cli/unidade-definir-colunas.md), com telas sintéticas
//! modeladas na estrutura da PEN1 5.0.4 (capturas privadas de 02/11/2026:
//! GET da tela e resposta do salvar que gravou). Ids e hashes fictícios.
use super::tests_alvo::{HASH_DO_SALVAR, tela_salvavel};
use super::*;

/// `inicializar()` como a tela serve: `V` vazio no GET, `1` depois de gravar.
fn script(sinal: &str) -> String {
    let destino = if sinal == "1" {
        "controlador.php?acao=procedimento_controlar&acao_origem=procedimento_configurar_detalhe&infra_sistema=100&infra_unidade_atual=200&infra_hash=8888888888888888888888888888888888888888888888888888888888888888"
    } else {
        ""
    };
    format!(
        "<script type=\"text/javascript\" charset=\"iso-8859-1\" >\n<!--//--><![CDATA[//><!--\n\n\n\
         function inicializar(){{\n  if ('{sinal}'=='1'){{\n    window.parent.location = '{destino}';\n    \
         self.setTimeout('infraFecharJanelaModal()',200);\n  }}\n}}\n\n\
         function OnSubmitForm() {{\n  return true;\n}}\n\n\n//--><!]]>\n</script>"
    )
}

/// A tela inteira na ordem da PEN1: `hdnInfraTipoPagina` logo após o
/// `<form>`, a barra com o link oculto de voltar e `sbmSalvar`, as 12
/// caixas (`onchange="validarNumeroCriterios(this)"`, sem `value`) e
/// `hdnInfraPrefixoCookie` fora do formulário.
fn tela_como_a_pen1(marcadas: &[ColunaDoDetalhe], sinal: &str) -> String {
    use std::fmt::Write as _;
    let mut caixas = String::new();
    for (i, coluna) in ColunaDoDetalhe::TODAS.into_iter().enumerate() {
        let campo = coluna.campo();
        let checked = if marcadas.contains(&coluna) {
            "  checked=\"checked\""
        } else {
            ""
        };
        write!(
            caixas,
            "\n    <div id=\"div{i}\" class=\"infraDivCheckbox infraAreaDados\" style=\"height:2.5em;display:block;\">\n      \
             <input type=\"checkbox\" id=\"{campo}\" name=\"{campo}\" class=\"infraCheckbox\" \
             onchange=\"validarNumeroCriterios(this)\"{checked}    tabindex=\"{}\" />\n      \
             <label id=\"lbl{i}\" for=\"{campo}\" accesskey=\"\" class=\"infraLabelCheckbox\">{}</label>\n    </div>\n",
            501 + i,
            coluna.rotulo()
        )
        .unwrap();
    }
    format!(
        "<!DOCTYPE html>\n<html lang=\"pt-br\" >\n<head >\n\
         <meta http-equiv=\"Content-Type\" content=\"text/html; charset=iso-8859-1\"/>\n\
         <title>SEI - Configurar Detalhe</title>\n{}\n</head>\n<body onload=\"inicializar();\"  >\n\
         <div id=\"divInfraBarraLocalizacao\" class=\"infraBarraLocalizacao\" tabindex=\"450\"><h1>Configurar Detalhe</h1></div>\n  \
         <form id=\"frmAcessoExternoCancelar\" method=\"post\" onsubmit=\"return OnSubmitForm();\" \
         action=\"controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_configurar_detalhe&amp;infra_sistema=100&amp;infra_unidade_atual=200&amp;infra_hash={HASH_DO_SALVAR}\">\n    \
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\"/>\n\
         <div id=\"divInfraBarraComandosSuperior\" class=\"infraBarraComandos\">\n\
         <a id=\"ancVoltarArvoreSuperior\" href=\"javascript:seiVoltarArvoreProcesso()\" class=\"btn\" style=\"display:none;\" tabindex=\"451\"><img src=\"/infra_css/svg/voltar.svg\"></a>\n\
         <button tabindex=\"451\" type=\"submit\" accesskey=\"S\" name=\"sbmSalvar\" value=\"Salvar\" class=\"infraButton\">\
         <span class=\"infraTeclaAtalho\">S</span>alvar</button>\n</div>\n{caixas}\n      </form>\n\
         <input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"SEI_SINTETICO\"/>\n\
         </body>\n</html>",
        script(sinal)
    )
}

/// Resposta sintética do salvar: a tela do Nível com o `inicializar()`.
pub(crate) fn resposta_do_salvar(
    publicadas: &[ColunaDoDetalhe],
    marcadas: &[ColunaDoDetalhe],
    sinal: &str,
) -> String {
    tela_salvavel(publicadas, marcadas, "").replacen(
        "</head>",
        &format!("{}</head>", script(sinal)),
        1,
    )
}

#[test]
fn post_do_salvar_repete_os_campos_do_navegador_na_tela_da_pen1() {
    use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};
    let servida = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_configurar_detalhe&acao_origem=procedimento_controlar&acao_retorno=procedimento_controlar&infra_sistema=100&infra_unidade_atual=200&infra_hash=7777777777777777777777777777777777777777777777777777777777777777";
    let html = tela_como_a_pen1(
        &[
            ColunaDoDetalhe::Atribuicao,
            ColunaDoDetalhe::TipoProcesso,
            ColunaDoDetalhe::Interessados,
        ],
        "",
    );
    let lido = ler_nivel_de_detalhe(&html).unwrap();
    assert_eq!(
        lido.marcadas(),
        [
            ColunaDoDetalhe::Atribuicao,
            ColunaDoDetalhe::TipoProcesso,
            ColunaDoDetalhe::Interessados
        ]
    );
    assert_eq!(conferir_controles_do_formulario(&html), Ok(()));
    let plano = planejar(&lido, &AlvoDasColunas::Todas).unwrap();
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let tela = TelaServida {
        url: servida,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let campos = campos_do_envio(&plano);
    let envio = derivar_submissao(
        &SALVAR_NIVEL_DE_DETALHE,
        &instalacao,
        &tela,
        "TESTE",
        &campos,
    )
    .unwrap();
    // O mesmo corpo, byte a byte, do POST que gravou na PEN1 em 02/11/2026.
    let caixas: Vec<String> = ColunaDoDetalhe::TODAS
        .iter()
        .map(|c| format!("{}=on", c.campo()))
        .collect();
    assert_eq!(
        String::from_utf8(envio.corpo().to_vec()).unwrap(),
        format!("hdnInfraTipoPagina=2&sbmSalvar=Salvar&{}", caixas.join("&"))
    );
    assert_eq!(envio.nomes_dos_campos().len(), 14);
    assert_eq!(
        envio.url(),
        format!(
            "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_configurar_detalhe&acao_origem=procedimento_configurar_detalhe&infra_sistema=100&infra_unidade_atual=200&infra_hash={HASH_DO_SALVAR}"
        )
    );
    assert_eq!(envio.referer(), servida);
    assert_eq!(envio.origem(), "https://sei.exemplo.gov.br");
}

#[test]
fn resposta_que_gravou_sinaliza_a_gravacao() {
    assert_eq!(
        sinal_do_salvar(&tela_como_a_pen1(&ColunaDoDetalhe::TODAS, "1")),
        SinalDoSalvar::GravacaoSinalizada
    );
    assert_eq!(
        sinal_do_salvar(&resposta_do_salvar(
            &ColunaDoDetalhe::TODAS,
            &ColunaDoDetalhe::TODAS,
            "1"
        )),
        SinalDoSalvar::GravacaoSinalizada
    );
}

#[test]
fn tela_como_no_get_volta_sem_sinal_de_gravacao() {
    assert_eq!(
        sinal_do_salvar(&tela_como_a_pen1(&[ColunaDoDetalhe::Atribuicao], "")),
        SinalDoSalvar::TelaDevolvidaSemSinal
    );
}

#[test]
fn resposta_sem_inicializar_na_forma_observada_nao_e_reconhecida() {
    let gravou = tela_como_a_pen1(&ColunaDoDetalhe::TODAS, "1");
    let duas = gravou.replacen("</head>", &format!("{}</head>", script("1")), 1);
    for html in [
        String::new(),
        "<html><body><script>infraFecharJanelaModal();</script></body></html>".to_owned(),
        tela_salvavel(&ColunaDoDetalhe::TODAS, &ColunaDoDetalhe::TODAS, ""),
        gravou.replace("if ('1'=='1')", "if ('S'=='1')"),
        gravou.replace("if ('1'=='1')", "if ('1'=='S')"),
        gravou.replace("if ('1'=='1')", "if (\"1\"==\"1\")"),
        gravou.replace("function inicializar(){", "function inicializar() {"),
        gravou.replace("if ('1'=='1')", "if ('1'"),
        duas,
    ] {
        assert_eq!(
            sinal_do_salvar(&html),
            SinalDoSalvar::NaoReconhecido,
            "{}",
            html.len()
        );
    }
}
