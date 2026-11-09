//! BDD-273-01/02 preexistentes: CLI real → TLS local → anti-IES → SQLite.
use super::*;

#[test]
fn bdd_273_01_cli_previa_sem_cancelar_documento() {
    let pasta = Pasta::nova();
    let saida = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "cancelar",
            DOC,
            "--motivo",
            "Teste ficticio",
        ],
        5, // pesquisa, tela, árvore, formulário; nenhum POST.
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let previa = json(&saida);
    assert_eq!(previa["numero"], DOC);
    assert_eq!(previa["motivo"], "Teste ficticio");
    assert_eq!(previa["confirmado"], false);
    assert_eq!(previa["alterado"], false);
    assert_eq!(previa["requisicoes"], 5);
}

#[test]
fn bdd_273_02_cli_um_cancelamento_com_releitura_e_sem_retry() {
    let pasta = Pasta::nova();
    let confirmacao = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "cancelar",
            DOC,
            "--motivo",
            "Teste ficticio",
            "--confirmar",
        ],
        10, // consulta inicial, formulário, POST único e nova abertura independente.
    );
    assert_eq!(
        confirmacao.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&confirmacao.stderr),
        String::from_utf8_lossy(&confirmacao.stdout),
    );
    let feito = json(&confirmacao);
    assert_eq!(feito["numero"], DOC);
    assert_eq!(feito["motivo"], "Teste ficticio");
    assert_eq!(feito["confirmado"], true);
    assert_eq!(feito["alterado"], true);
    assert_eq!(feito["reconciliado_por_leitura"], true);
    assert_eq!(feito["requisicoes"], 10);
    assert!(feito.get("falha_do_envio").is_none());
}

/// A confirmação da CLI altera o mesmo SQLite usado pelas rotas do anti-IES.
/// Prova técnica complementar ao BDD-273-02: a árvore posterior deixa de
/// oferecer cancelar, e uma nova tentativa não substitui o motivo original.
#[test]
fn bdd_273_02_estado_cancelado_persiste_entre_cli_arvore_e_formulario() {
    use iesei::cancelamento_documento::{self, Situacao};

    let pasta = Pasta::nova();
    let motivo = "Cancelamento ficticio original";
    let saida = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "cancelar",
            DOC,
            "--motivo",
            motivo,
            "--confirmar",
        ],
        10,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "a escrita inicial não foi confirmada: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    assert_eq!(json(&saida)["alterado"], true);

    // Esta leitura do SQLite não passa pelo renderizador HTML nem pelo
    // parser da CLI: é uma expectativa independente do efeito persistido.
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    let estado = || cancelamento_documento::consultar(&banco, &origem, CENARIO, "42", "7").unwrap();
    assert_eq!(
        estado(),
        Some(Situacao::Cancelado {
            motivo: motivo.into()
        })
    );

    let arvore = anti_ies::documento_arvore::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_visualizar&id_documento=42&id_procedimento=7",
        &banco,
        &origem,
        CENARIO,
    );
    assert_eq!(arvore.estado, anti_ies::Estado::Ok);
    assert!(arvore.corpo.contains("documento_cancelado.svg"));
    assert!(arvore.corpo.contains(motivo));
    assert!(!arvore.corpo.contains("Cancelar Documento"));

    let alvo = "/sei/controlador.php?acao=documento_cancelar&id_procedimento=7&id_documento=42";
    let formulario =
        anti_ies::documento_cancelar::responder("GET", alvo, "", &banco, &origem, CENARIO);
    assert_eq!(formulario.estado, anti_ies::Estado::MetodoNaoPermitido);

    // Mesmo um POST sintético direto não cancela duas vezes nem troca o
    // motivo, porque o servidor consulta o estado persistido primeiro.
    let repeticao = anti_ies::documento_cancelar::responder(
        "POST",
        alvo,
        "hdnInfraTipoPagina=2&sbmSalvar=Salvar&txaMotivo=Outro",
        &banco,
        &origem,
        CENARIO,
    );
    assert_eq!(repeticao.estado, anti_ies::Estado::MetodoNaoPermitido);
    assert_eq!(
        estado(),
        Some(Situacao::Cancelado {
            motivo: motivo.into()
        })
    );
}
/// Regressão técnica do BDD-273-01/02: o texto da CLI é opaco. O form
/// urlencoded deve conservar Unicode, espaços, quebra de linha e caracteres
/// reservados, sem normalização pelo IES nem pelo anti-IES.
#[test]
fn bdd_273_02_motivo_opaco_sobrevive_previa_post_e_sqlite() {
    use iesei::cancelamento_documento::{self, Situacao};

    let pasta = Pasta::nova();
    let motivo = "  Revisão\nnecessária & conferência=1 + \"literal\"  ";
    let args = [
        "--json", "documento", "cancelar", DOC, "--motivo", motivo,
    ];
    let previa = executar(&pasta, &args, 5);
    assert_eq!(
        previa.status.code(),
        Some(0),
        "prévia falhou: {}",
        String::from_utf8_lossy(&previa.stderr)
    );
    assert_eq!(json(&previa)["motivo"], motivo);
    assert_eq!(json(&previa)["alterado"], false);

    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let banco = Connection::open(&caminho).unwrap();
    assert_eq!(
        cancelamento_documento::consultar(&banco, &origem, CENARIO, "42", "7").unwrap(),
        Some(Situacao::Disponivel)
    );
    drop(banco);

    let confirmado = executar(
        &pasta,
        &[
            "--json", "documento", "cancelar", DOC, "--motivo", motivo,
            "--confirmar",
        ],
        10,
    );
    assert_eq!(
        confirmado.status.code(),
        Some(0),
        "confirmação falhou: {}",
        String::from_utf8_lossy(&confirmado.stderr)
    );
    let resultado = json(&confirmado);
    assert_eq!(resultado["motivo"], motivo);
    assert_eq!(resultado["alterado"], true);
    assert_eq!(resultado["reconciliado_por_leitura"], true);
    assert_eq!(resultado["requisicoes"], 10);

    let banco = Connection::open(&caminho).unwrap();
    assert_eq!(
        cancelamento_documento::consultar(&banco, &origem, CENARIO, "42", "7").unwrap(),
        Some(Situacao::Cancelado { motivo: motivo.into() })
    );
}
