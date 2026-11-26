//! Replay de conformidade do acervo público para a lista de blocos
//! (`bloco_assinatura_listar`, `bloco_interno_listar`, `bloco_reuniao_listar`):
//! passa as respostas gravadas pelo leitor de produção `ler_lista`, sem
//! afrouxar asserts. Refs #797.
use super::{TipoBloco, ler_lista};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::tela_html::HtmlServido;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

fn tipo_da_operacao(operacao: &str) -> Option<TipoBloco> {
    match operacao {
        "bloco_assinatura_listar" => Some(TipoBloco::Assinatura),
        "bloco_interno_listar" => Some(TipoBloco::Interno),
        "bloco_reuniao_listar" => Some(TipoBloco::Reuniao),
        _ => None,
    }
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        if r.status != 200 {
            continue;
        }
        let Some(tipo) = tipo_da_operacao(r.operacao) else {
            continue;
        };
        let classe = match html_da_resposta(r.interacao) {
            Ok(html) => match ler_lista(&HtmlServido::de_teste(&html), tipo) {
                Ok(lista) if lista.lista.itens.is_empty() => "vazia".to_owned(),
                Ok(_) => "lida".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            },
            Err(erro) => format!("HTML ilegível: `{erro}`"),
        };
        leituras.push((tipo.rotulo(), classe));
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_da_lista_de_blocos_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "bloco_assinatura_listar",
        "bloco_interno_listar",
        "bloco_reuniao_listar",
    ];
    let lidos = percorrer(&operacoes, |_, instalacao, json| {
        for (tipo, classe) in replay(json) {
            *contagem
                .entry((tipo, classe))
                .or_default()
                .entry(instalacao.to_owned())
                .or_default() += 1;
        }
    });
    let mut s = format!("{lidos} cassettes lidas.\n\n| Tipo | Classe |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push_str("\n| --- | --- |");
    s.push_str(&" --- |".repeat(INSTALACOES.len()));
    for ((tipo, classe), por) in &contagem {
        let _ = write!(s, "\n| {tipo} | {classe} |");
        for i in INSTALACOES {
            let _ = write!(s, " {} |", por.get(i).copied().unwrap_or(0));
        }
    }
    s.push('\n');
    publicar(&s);

    let recusas: Vec<_> = contagem
        .keys()
        .filter(|(_, classe)| classe.starts_with('`') || classe.starts_with("HTML"))
        .collect();
    assert!(
        recusas.is_empty(),
        "leitores recusaram a gravação: {recusas:?}"
    );
    let total: u64 = contagem.values().flat_map(|por| por.values()).sum();
    assert!(total > 0, "nenhuma lista de blocos lida");
}

// Primeira fatia #876: confirmar o contrato de criação a partir dos bytes
// originais, não do Location sintético dos testes do Todos.
#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE; sem rede"]
fn replay_da_criacao_de_bloco_de_assinatura_no_acervo() {
    use crate::consulta_do_controlador::ConsultaDoControlador;
    use crate::formulario_web::consulta_do_controlador;
    use crate::tela_html::arvore;

    let mut vistos = BTreeMap::new();
    let mut grupos = BTreeMap::new();
    percorrer(&["bloco_assinatura_cadastrar"], |nome, instalacao, json| {
        let campo = match nome {
            "bloco-ciclo-ufrj--9119d6015d02" => "txtDescricao",
            "bloco-ciclo-pen3--de556c70c2ad" => "txaDescricao",
            _ => return,
        };
        let respostas = gravacao::respostas(json);
        let formulario = respostas
            .iter()
            .find(|r| {
                r.operacao == "bloco_assinatura_cadastrar" && r.metodo == "GET" && r.status == 200
            })
            .expect("GET do cadastro não gravado");
        let html = HtmlServido::de_teste(
            &html_da_resposta(formulario.interacao).expect("HTML do cadastro"),
        );
        let dom = arvore(&html);
        let forms: Vec<_> = dom.com_id("frmBlocoCadastro").collect();
        let [form] = forms.as_slice() else {
            panic!("{instalacao}: cadastro não único");
        };
        assert_eq!(form.nome(), "form");
        let descricoes: Vec<_> = form
            .elementos("textarea")
            .filter_map(|no| no.atributo("name"))
            .filter(|n| matches!(*n, "txtDescricao" | "txaDescricao"))
            .collect();
        assert_eq!(descricoes, [campo], "{instalacao}: descrição");
        assert_eq!(
            crate::bloco_cli::criar::campo_descricao(&html).expect("leitor de produção"),
            campo
        );
        assert!(
            form.descendentes()
                .any(|no| no.atributo("name") == Some("sbmCadastrarBloco")),
            "{instalacao}: botão Salvar"
        );

        // O builder Rust só emite campos de usuário que a operação fornece.
        // Observar o default do grupo antes de projetar a submissão.
        let grupo = grupo_padrao_do_cadastro(*form);
        assert_eq!(grupo.as_deref(), Some("null"), "{instalacao}");
        assert_eq!(
            crate::bloco_cli::criar::grupo_padrao(&html).expect("leitor de produção"),
            grupo
        );
        let classe_grupo = match grupo.as_deref() {
            None => "ausente",
            Some("") => "vazio",
            Some("null") => "null",
            Some(_) => "outro",
        };
        grupos.insert(instalacao.to_owned(), classe_grupo);

        conferir_derivacao_do_cadastro(
            json,
            &respostas,
            formulario,
            &html,
            campo,
            grupo.as_deref().expect("grupo publicado"),
        );

        let post = respostas
            .iter()
            .find(|r| {
                r.passo > formulario.passo
                    && r.operacao == "bloco_assinatura_cadastrar"
                    && r.metodo == "POST"
                    && r.status == 302
            })
            .expect("POST 302 de criação não gravado");
        let url = gravacao::url_sem_consulta(json, post.interacao);
        let consulta = gravacao::localizacao(post.interacao)
            .and_then(|local| consulta_do_controlador(local, &url))
            .and_then(|texto| ConsultaDoControlador::ler(texto).ok())
            .expect("Location original não recuperado");
        assert_eq!(consulta.acao().ok(), Some("bloco_assinatura_listar"));
        let id = consulta.unico("id_bloco").expect("id do bloco no Location");
        assert!(!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));

        let (antes, depois) = listas_da_criacao(&respostas, formulario.passo, post.passo);
        let antes = ler_lista_gravada(antes.expect("lista anterior ao cadastro"));
        assert!(
            !antes.itens.iter().any(|b| b.numero == id),
            "{instalacao}: identidade já observada antes da criação"
        );
        let depois = ler_lista_gravada(depois.expect("lista após POST, antes de outro POST"));
        assert!(
            depois.itens.iter().any(|b| {
                b.numero == id && b.estado == crate::bloco_estado::EstadoDoBloco::Gerado
            }),
            "{instalacao}: identidade e estado Gerado não observados após criação"
        );
        *vistos.entry(instalacao.to_owned()).or_insert(0usize) += 1;
    });
    assert_eq!(vistos.get("ufrj"), Some(&1));
    assert_eq!(vistos.get("pen3"), Some(&1));
    let mut relatorio = String::new();
    for (instalacao, classe) in grupos {
        let _ = writeln!(relatorio, "{instalacao}: grupo padrão {classe}");
    }
    publicar(&relatorio);
}

fn conferir_derivacao_do_cadastro(
    json: &serde_json::Value,
    respostas: &[gravacao::Resposta<'_>],
    formulario: &gravacao::Resposta<'_>,
    html: &HtmlServido,
    campo: &str,
    grupo: &str,
) {
    // Exercita o construtor real sobre a tela original, com descrição
    // sintética: os cards não atestam o corpo histórico da requisição.
    let controle = respostas
        .iter()
        .rev()
        .find(|r| {
            r.passo < formulario.passo
                && r.operacao == "procedimento_controlar"
                && r.metodo == "GET"
                && r.status == 200
        })
        .expect("Controle anterior ao cadastro");
    let html_controle =
        HtmlServido::de_teste(&html_da_resposta(controle.interacao).expect("HTML do Controle"));
    let unidade = crate::controle_reconhecido::unidade_do_controle(&html_controle)
        .expect("unidade do Controle original");
    let origem = gravacao::instalacao(json).expect("instalação da cassette");
    let url_cadastro = gravacao::url_sem_consulta(json, formulario.interacao);
    let codificacao =
        crate::pagina_do_controle::codificacao_da_tela(&gravacao::mime(formulario.interacao))
            .expect("charset do cadastro");
    let tela = crate::formulario_web::TelaServida {
        url: &url_cadastro,
        html,
        codificacao,
    };
    let submissao =
        crate::bloco_cli::criar::derivar_cadastro(&origem, &tela, &unidade, "Teste IES", grupo)
            .expect("derivação do cadastro original");
    for nome in [campo, "selGrupoBloco", "sbmCadastrarBloco"] {
        assert!(
            submissao.nomes_dos_campos().iter().any(|n| n == nome),
            "campo não derivado: {nome}"
        );
    }
}

// A testemunha anterior é a última lista GET antes do cadastro; a
// posterior é a primeira lista GET antes de outra submissão. Não
// atribuir à criação uma linha vista somente no fim de todo o roteiro.
fn listas_da_criacao<'a, 'j>(
    respostas: &'a [gravacao::Resposta<'j>],
    cadastro: usize,
    post: usize,
) -> (
    Option<&'a gravacao::Resposta<'j>>,
    Option<&'a gravacao::Resposta<'j>>,
) {
    let e_lista = |r: &&gravacao::Resposta<'_>| {
        r.operacao == "bloco_assinatura_listar" && r.metodo == "GET" && r.status == 200
    };
    let antes = respostas
        .iter()
        .rev()
        .filter(|r| r.passo < cadastro)
        .find(e_lista);
    let depois = respostas
        .iter()
        .filter(|r| r.passo > post)
        .take_while(|r| r.metodo != "POST")
        .find(e_lista);
    (antes, depois)
}

fn ler_lista_gravada(r: &gravacao::Resposta<'_>) -> super::ListaDeBlocos {
    let html = html_da_resposta(r.interacao).expect("HTML da lista");
    ler_lista(&HtmlServido::de_teste(&html), TipoBloco::Assinatura)
        .expect("lista de assinatura")
        .lista
}

#[test]
fn testemunha_da_criacao_nao_pula_para_lista_depois_de_outra_escrita() {
    use serde_json::json;
    let lista = json!({
        "request": {"operation": "bloco_assinatura_listar", "method": "GET"},
        "response": {"status_code": 200}
    });
    let cadastro = json!({
        "request": {"operation": "bloco_assinatura_cadastrar", "method": "GET"},
        "response": {"status_code": 200}
    });
    let post = json!({
        "request": {"operation": "bloco_assinatura_cadastrar", "method": "POST"},
        "response": {"status_code": 302}
    });
    let outro = json!({
        "request": {"operation": "bloco_excluir", "method": "POST"},
        "response": {"status_code": 302}
    });
    let positivo = json!({"interactions": [lista.clone(), cadastro.clone(), post.clone(), lista.clone(), outro.clone(), lista.clone()]});
    let respostas = gravacao::respostas(&positivo);
    let (antes, depois) = listas_da_criacao(&respostas, 1, 2);
    assert_eq!(antes.map(|r| r.passo), Some(0));
    assert_eq!(depois.map(|r| r.passo), Some(3));

    let negativo = json!({"interactions": [lista.clone(), cadastro, post, outro, lista]});
    let respostas = gravacao::respostas(&negativo);
    let (antes, depois) = listas_da_criacao(&respostas, 1, 2);
    assert_eq!(antes.map(|r| r.passo), Some(0));
    assert!(depois.is_none());
}

// Oráculo de replay para o select simples do cadastro. Sem seleção explícita,
// a primeira opção habilitada é o default; vazio não é ausência do controle.
fn grupo_padrao_do_cadastro(form: crate::tela_html::Elemento<'_>) -> Option<String> {
    let mut grupos = form
        .elementos("select")
        .filter(|no| no.atributo("name") == Some("selGrupoBloco"));
    let grupo = grupos.next()?;
    assert!(grupos.next().is_none(), "grupo repetido no cadastro");
    if grupo.presente("disabled") {
        return None;
    }
    assert!(!grupo.presente("multiple"), "grupo não é seleção simples");
    assert!(
        grupo.elementos("optgroup").next().is_none(),
        "grupo com estrutura ainda não observada"
    );
    let opcoes: Vec<_> = grupo.elementos("option").collect();
    let marcadas: Vec<_> = opcoes
        .iter()
        .copied()
        .filter(|opcao| opcao.presente("selected"))
        .collect();
    let opcao = match marcadas.as_slice() {
        [] => opcoes.iter().copied().find(|o| !o.presente("disabled")),
        [opcao] => Some(*opcao),
        _ => panic!("grupo com mais de uma seleção no cadastro"),
    }?;
    if opcao.presente("disabled") {
        return None;
    }
    Some(
        opcao
            .atributo("value")
            .map_or_else(|| opcao.texto().trim().to_owned(), str::to_owned),
    )
}

#[test]
fn grupo_padrao_do_cadastro_distingue_valor_vazio_e_ausencia() {
    let casos = [
        (
            "<select name='selGrupoBloco'><option value='null'>Nenhum</option></select>",
            Some("null"),
        ),
        (
            "<select name='selGrupoBloco'><option value='null'>Nenhum</option><option value='' selected>Todos</option></select>",
            Some(""),
        ),
        (
            "<select name='selGrupoBloco'><option value='null'>Nenhum</option><option value='4' selected>Grupo</option></select>",
            Some("4"),
        ),
        ("<textarea name='txtDescricao'></textarea>", None),
        (
            "<select name='selGrupoBloco'><option>Nenhum</option></select>",
            Some("Nenhum"),
        ),
    ];
    for (controles, esperado) in casos {
        let html = HtmlServido::de_teste(&format!("<form>{controles}</form>"));
        let dom = crate::tela_html::arvore(&html);
        let form = dom.elementos("form").next().unwrap();
        assert_eq!(grupo_padrao_do_cadastro(form).as_deref(), esperado);
    }
}
