//! Fluxos inteiros com transporte roteirizado, sem rede ou dados reais.
//! Oráculo: Todos src/todos/documentos.py @323eb01; forma Web 4.0.
//! O orçamento inclui abertura, escolha, cadastro, escrita e verificação.
use super::*;
use crate::arvore_processo::tests::no_documento;
use crate::processo_concluir::tests_apoio as a;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_escolha() -> String {
    format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=10&infra_hash={}",
        "e".repeat(64)
    )
}

fn link_cadastro() -> String {
    format!(
        "controlador.php?acao=documento_gerar&id_procedimento=10&infra_hash={}",
        "f".repeat(64)
    )
}

fn url(relativo: &str) -> String {
    format!("{}/{}", a::ORIGEM, relativo)
}

fn arvore_inicial() -> String {
    a::arvore(&[a::UNIDADE], &[acao_da_barra(&link_escolha())])
}

fn lista_de_tipos() -> String {
    let href = link_cadastro().replace('&', "&amp;");
    format!(
        r#"<html><table id="tblSeries"><tr><td><a href="{href}">Despacho</a></td></tr></table></html>"#
    )
}

fn cadastro_com_script(script: &str) -> String {
    let action = link_cadastro().replace('&', "&amp;");
    format!(
        r#"<html><head><script>{script}</script></head><body>
        <label id="lblSerieTitulo">Despacho</label>
        <form id="frmDocumentoCadastro" method="post" action="{action}">
          <input type="hidden" name="hdnIdSerie" value="291">
          <input type="hidden" name="hdnFlagDocumentoCadastro" value="1">
          <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
          <input type="text" id="txtDescricao" name="txtDescricao" value="">
        </form>
        <button type="button" id="btnSalvar" name="btnSalvar"
          onclick="confirmarDados()">Salvar</button></body></html>"#
    )
}

fn cadastro() -> String {
    // Forma de submeter() servida no SEI 4.0.9 (Todos @323eb01).
    cadastro_com_script(
        r"function confirmarDados() {
  if (OnSubmitForm()) {
    submeter();
  }
}
function submeter() {
  desabilitarBotaoSalvar(true);
  document.getElementById('hdnFlagDocumentoCadastro').value = '2';
  document.getElementById('frmDocumentoCadastro').submit();
}",
    )
}

fn arvore_verificadora(novos: usize) -> String {
    let mut nos = vec![
        crate::arvore_processo::tests::no_processo(0, a::ID, a::NUP),
        no_documento(1, "21", a::ID, "0000101"),
    ];
    if novos >= 1 {
        nos.push(no_documento(2, "22", a::ID, "0000102"));
    }
    if novos >= 2 {
        nos.push(no_documento(3, "23", a::ID, "0000103"));
    }
    crate::arvore_processo::tests::arvore(&nos)
}

fn executar(
    confirmar: bool,
    extra: Vec<a::Resposta>,
) -> (Result<PreviaCriacao, ErroPreviaCriacao>, Vec<a::Pedido>) {
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(cadastro()));
    respostas.extend(extra);
    a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: Some("Ensaio"),
            nivel: Some("publico"),
        };
        executar_criacao(sessao, &pedido, indice, rede, confirmar)
    })
}

fn pedidos_antes_da_escrita() -> Vec<a::Pedido> {
    let mut pedidos = a::pedidos_da_abertura();
    pedidos.push(a::get(&url(&link_escolha())));
    pedidos.push(a::get(&url(&link_cadastro())));
    pedidos
}

fn pedido_criador() -> a::Pedido {
    a::post(
        &url(&link_cadastro()),
        "hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Ensaio",
    )
}

#[test]
fn previa_completa_so_cinco_gets_e_nenhum_post() {
    let (resultado, pedidos) = executar(false, vec![]);
    let previa = match resultado {
        Ok(previa) => previa,
        Err(erro) => panic!("prévia falhou: {}", erro.subcodigo()),
    };
    assert!(!previa.confirmado);
    assert_eq!(previa.numero_sei, None);
    assert_eq!(previa.requisicoes, 5);
    assert_eq!(pedidos, pedidos_antes_da_escrita());
}

#[test]
fn confirmar_unico_post_releitura_e_numero_sei_com_sete_requisicoes() {
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::ok("<html>salvo</html>".to_owned()),
            a::ok(arvore_verificadora(1)),
        ],
    );
    let confirmacao = match resultado {
        Ok(confirmacao) => confirmacao,
        Err(erro) => panic!("confirmação sintética recusada: {}", erro.subcodigo()),
    };
    assert!(confirmacao.confirmado);
    assert_eq!(confirmacao.numero_sei.as_deref(), Some("0000102"));
    assert_eq!(confirmacao.requisicoes, 7);
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

#[test]
fn dois_documentos_apos_post_sao_incerteza_sem_retry() {
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::ok("<html>salvo</html>".to_owned()),
            a::ok(arvore_verificadora(2)),
        ],
    );
    assert_eq!(
        resultado.err().map(|erro| erro.subcodigo()),
        Some("efeito_incerto_na_criacao_de_documento")
    );
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

fn executar_com_cadastro_sintetico(
    html_cadastro: String,
) -> (Result<PreviaCriacao, ErroPreviaCriacao>, Vec<a::Pedido>) {
    let ambiente = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(html_cadastro));
    a::rodar_com(&ambiente, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: Some("Ensaio"),
            nivel: Some("publico"),
        };
        executar_criacao(sessao, &pedido, indice, rede, true)
    })
}

#[test]
fn funcao_submeter_observada_no_sei_409_publica_sinalizador() {
    // O controle positivo usa a forma completa observada, nunca uma
    // atribuicao sintetica isolada.
    assert!(cadastro_publica_script_de_criacao(&cadastro()));
}

#[test]
fn atribuicao_isolada_nao_comprova_caminho_de_submissao_e_recusa_post() {
    // A forma observada em Todos documento_gerar.html @323eb01
    // exige a funcao submeter(), a desabilitacao e o submit do formulario.
    // Uma atribuicao isolada pode existir numa tela que nunca envia.
    // Nao executar JS; recusar ANTES da escrita, com cinco GET exatos.
    let (resultado, pedidos) = executar_com_cadastro_sintetico(cadastro_com_script(
        "document.getElementById('hdnFlagDocumentoCadastro').value = '2';",
    ));
    assert_eq!(
        resultado.err().map(|erro| erro.subcodigo()),
        Some("submissao_do_documento_nao_observada"),
    );
    assert_eq!(
        pedidos,
        pedidos_antes_da_escrita(),
        "cinco GET de previa, nenhum POST criador",
    );
}

#[test]
fn script_inerte_ou_duplicado_nao_autoriza_post_criador() {
    // O HTML de treinamento não prova uma instrução executável quando a
    // sequência aparece em template, JSON, comentário ou script duplicado.
    // O teste não executa JavaScript nem faz requisições ao SEI.
    let casos = [
        cadastro()
            .replace("<script>", "<template>")
            .replace("</script>", "</template>"),
        cadastro().replace("<script>", "<script type=\"application/json\">"),
        cadastro()
            .replace("<script>", "<script>/*")
            .replace("</script>", "*/</script>"),
        cadastro().replace(
            "</script>",
            "</script><script>document.getElementById('hdnFlagDocumentoCadastro').value = '2';</script>",
        ),
        // Script remoto ignora o corpo inline: não é prova de envio.
        cadastro().replace("<script>", "<script src=\"/static/externo.js\">"),
        // A linha aparece apenas como texto de um template literal JS
        // multilinha, sem instrução executada pelo navegador.
        cadastro().replace(
            "<script>",
            "<script>const amostra = `\n",
        ).replace(
            "</script>",
            "\n`;\n</script>",
        ),
        // Atribuição em ramo comprovadamente inatingível: nenhuma escrita.
        cadastro()
            .replace("<script>", "<script>if (false) {\n")
            .replace("</script>", "\n}</script>"),
        // Uma função arbitrária que não integra o caminho de salvar não
        // comprova a operação, ainda que seu corpo contenha a instrução.
        cadastro()
            .replace("<script>", "<script>function exemploNaoChamado() {\n")
            .replace("</script>", "\n}</script>"),
        // Mesmo uma função chamada submeter não autoriza instrução
        // dentro de seu próprio ramo não alcançável.
        cadastro()
            .replace("<script>", "<script>function submeter() {\nif (false) {\n")
            .replace("</script>", "\n}\n}</script>"),
    ];
    for html in casos {
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada")
        );
        assert_eq!(
            pedidos,
            pedidos_antes_da_escrita(),
            "somente os cinco GETs de prévia; nenhum POST criador"
        );
    }
}

#[test]
fn salvar_sem_botao_observado_ou_com_acionador_divergente_recusa_antes_do_post() {
    // Captura Todos documento_gerar.html @323eb01: linhas 810 e 1061
    // publicam dois botoes btnSalvar habilitados e onclick=confirmarDados().
    // Nao basta uma funcao submeter() morta no script para autorizar escrita.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(original.contains(botao));
    let casos = [
        original.replace(botao, ""),
        original.replace("onclick=\"confirmarDados()\"", "onclick=\"outraAcao()\""),
        original.replace("type=\"button\"", "type=\"submit\""),
        original.replace("onclick=\"confirmarDados()\"", "disabled onclick=\"confirmarDados()\""),
        // Um segundo botao divergente nao pode ser ignorado so porque
        // o primeiro continua valido.
        original.replace(
            "</body>",
            "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\" onclick=\"outraAcao()\">Salvar</button></body>",
        ),
    ];
    for html in casos {
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada")
        );
        assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET, 0 POST criador");
    }
}

#[test]
fn dois_botoes_salvar_identicos_sao_permitidos_na_captura() {
    // Topo e rodape da captura 4.0.9 repetem intencionalmente btnSalvar.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(cadastro_publica_script_de_criacao(
        &original.replace("</body>", &format!("{botao}</body>"))
    ));
}

#[test]
fn salvar_sem_cadeia_de_confirmacao_observada_recusa_antes_do_post() {
    // Todos tests/paginas_reais/documento_gerar.html @323eb01,
    // linhas 615-633: btnSalvar -> confirmarDados() ->
    // if (OnSubmitForm()) -> submeter() -> frmDocumentoCadastro.submit().
    // Ter o botao e a funcao submeter separados nao prova essa cadeia.
    let original = cadastro();
    let confirmacao =
        "function confirmarDados() {\n  if (OnSubmitForm()) {\n    submeter();\n  }\n}\n";
    assert!(original.contains(confirmacao));
    let casos = [
        original.replace(confirmacao, ""),
        original.replace("function confirmarDados()", "function naoChamado()"),
        original.replace("if (OnSubmitForm()) {", "if (false) {"),
        original.replace("    submeter();", "    outraAcao();"),
        original.replace(
            "  if (OnSubmitForm()) {\n    submeter();\n  }",
            "  submeter();",
        ),
        original.replace(
            "</script>",
            "</script><script>function confirmarDados() {}</script>",
        ),
    ];
    for html in casos {
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada"),
            "sem cadeia ativa comprovada, a confirmação deve falhar fechada"
        );
        assert_eq!(
            pedidos,
            pedidos_antes_da_escrita(),
            "orçamento exato: 5 GET de prévia e 0 POST criador"
        );
    }
}

#[test]
fn script_alheio_com_regex_literal_nao_invalida_cadeia_salvar() {
    // Observação UnB 42209629903: o script ativo que contém a cadeia
    // de Salvar possui centenas de linhas e muitas funções alheias.
    // A regex abaixo é sintética, NÃO atribuída à captura real.
    // A evidência das três funções continua literal e não condicionada.
    let html = cadastro().replace(
        "</script>",
        "</script><script>const chaves = /\\{/;</script>",
    );
    assert!(
        cadastro_publica_script_de_criacao(&html),
        "script de apoio sem função de criação não deve invalidar a cadeia ativa"
    );
}

#[test]
fn regex_com_falsa_definicao_nao_comprova_criacao() {
    // Texto de regex não pode substituir as funções de Salvar.
    let html = cadastro_com_script(
        r"const falso = /function submeter\(\) \{/;
const igualmenteFalso = /function confirmarDados\(\) \{/;",
    );
    let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
    assert_eq!(
        resultado.err().map(|erro| erro.subcodigo()),
        Some("submissao_do_documento_nao_observada")
    );
    assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET; 0 POST criador");
}

#[test]
fn template_inerte_nao_autoriza_post_criador_ao_conter_script_ou_botao() {
    // Semantica de HTMLTemplateElement: o conteudo nao integra a arvore
    // ativa; nao autoriza escrita mesmo se copiar a forma literal de Salvar.
    // Caso adversarial sintético, nao atribuido ao cadastro real da UnB.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(original.contains("<script>"));
    assert!(original.contains(botao));

    let script_inerte = original
        .replace("<script>", "<template><script>")
        .replace("</script>", "</script></template>");
    let botao_inerte = original.replace(botao, &format!("<template>{botao}</template>"));
    for html in [script_inerte, botao_inerte] {
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada"),
            "script e botao em template nao sao controles ativos"
        );
        assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET, 0 POST criador");
    }
}

#[test]
fn html_literal_dentro_de_script_nao_oculta_a_cadeia_de_salvar() {
    // Diagnóstico em TREINAMENTO UnB #246 / run 42222453651:
    // tl::inner_text perdeu o corpo do script extenso (15 bytes).
    // O literal HTML auxiliar é sintético: não pertence à captura privada.
    let html = cadastro().replace(
        "<script>",
        "<script>\nconst fragmento = '<div><span>auxiliar</span></div>';\n",
    );
    assert!(
        cadastro_publica_script_de_criacao(&html),
        "script ativo preserva a cadeia literal mesmo com HTML em strings JS"
    );
}

#[test]
fn atributo_com_maior_que_na_abertura_do_script_preserva_a_fonte() {
    // HTML validado pelo DOM: '>' dentro de atributo nao fecha <script>.
    let html = cadastro().replace(
        "<script>",
        "<script type=\"text/javascript\" data-exemplo=\"um > dois\">",
    );
    assert!(cadastro_publica_script_de_criacao(&html));
}

#[test]
fn fechamento_de_script_dentro_de_literal_interrompe_cadeia() {
    // Pela tokenizacao HTML, </script> encerra o elemento mesmo se os
    // caracteres estiverem escritos numa string JavaScript.
    // As funcoes seguintes NAO sao codigo daquele script.
    let html = cadastro().replace("<script>", "<script>\nconst fragmento = '</script>';\n");
    let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
    assert_eq!(
        resultado.err().map(|erro| erro.subcodigo()),
        Some("submissao_do_documento_nao_observada")
    );
    assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET, 0 POST");
}
