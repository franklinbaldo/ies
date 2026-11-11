//! Abertura, permissões, falhas que desligam, concorrência, identidade e
//! capacidade da instalação. Diretórios temporários, sem rede.
use super::*;
use crate::instalacao::Instalacao;
use std::fs;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn pasta(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ies-indice-{nome}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    definir_modo(&dir, 0o700);
    dir
}

/// Modo Unix do arquivo; fora do Unix não há modo e nada muda.
pub(crate) fn definir_modo(caminho: &Path, modo: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(caminho, fs::Permissions::from_mode(modo)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = (caminho, modo);
}

pub(super) fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

#[cfg(unix)]
fn modo(caminho: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(caminho).unwrap().permissions().mode() & 0o777
}

#[cfg(unix)]
#[test]
fn cria_diretorio_0700_e_banco_0600_com_wal_herdando() {
    let raiz = pasta("permissoes");
    let caminho = raiz.join("sub").join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_processo(&instalacao(), "N1", "10");
    assert!(indice.ligado());
    assert_eq!(
        indice.id_por_nup(&instalacao(), "N1").as_deref(),
        Some("10")
    );
    assert_eq!(modo(&raiz.join("sub")), 0o700);
    assert_eq!(modo(&caminho), 0o600);
    for sufixo in ["-wal", "-shm"] {
        let lateral = PathBuf::from(format!("{}{sufixo}", caminho.display()));
        if lateral.exists() {
            assert_eq!(modo(&lateral), 0o600, "{sufixo}");
        }
    }
}

#[cfg(unix)]
#[test]
fn banco_link_simbolico_desliga_sem_tocar_no_alvo() {
    let raiz = pasta("symlink");
    let alvo = raiz.join("alvo");
    fs::write(&alvo, b"").unwrap();
    definir_modo(&alvo, 0o600);
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    std::os::unix::fs::symlink(&alvo, &caminho).unwrap();
    let indice = IndiceLocal::em(caminho);
    indice.registrar_processo(&instalacao(), "N1", "10");
    assert_eq!(indice.id_por_nup(&instalacao(), "N1"), None);
    assert!(!indice.ligado());
    assert_eq!(fs::metadata(&alvo).unwrap().len(), 0);
}

#[cfg(unix)]
#[test]
fn diretorio_por_link_simbolico_liga_e_cria_os_niveis_no_diretorio_real() {
    // Ancestral link (`/home` → `/var/home`) não desliga o índice.
    let raiz = pasta("dir-symlink");
    let real = raiz.join("real");
    fs::create_dir(&real).unwrap();
    definir_modo(&real, 0o700);
    std::os::unix::fs::symlink(&real, raiz.join("atalho")).unwrap();
    let indice = IndiceLocal::em(raiz.join("atalho").join("novo").join(NOME_DO_ARQUIVO));
    indice.registrar_entrada_por_id(&instalacao(), true);
    assert!(indice.ligado());
    assert_eq!(
        indice.entrada_por_id(&instalacao()),
        Some(AceiteDaEntrada::Aceita)
    );
    assert_eq!(modo(&real.join("novo")), 0o700);
    assert_eq!(modo(&real.join("novo").join(NOME_DO_ARQUIVO)), 0o600);
}

#[cfg(unix)]
#[test]
fn wal_link_simbolico_desliga() {
    let raiz = pasta("wal-symlink");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let wal = PathBuf::from(format!("{}-wal", caminho.display()));
    let _ = fs::remove_file(&wal);
    std::os::unix::fs::symlink(raiz.join("outro"), &wal).unwrap();
    let indice = IndiceLocal::em(caminho);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
}

#[cfg(unix)]
#[test]
fn diretorio_aberto_a_outros_desliga_e_nao_cria_banco() {
    let raiz = pasta("dir-aberto");
    definir_modo(&raiz, 0o755);
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_entrada_por_id(&instalacao(), true);
    assert!(!indice.ligado());
    assert!(!caminho.exists());
}

/// O índice é do IES: o nível novo não nasce num diretório gravável
/// pelo grupo ou por outros sem sticky bit (o índice desliga).
#[cfg(unix)]
#[test]
fn nivel_novo_em_diretorio_gravavel_por_outros_desliga_sem_criar() {
    let raiz = pasta("pai-aberto");
    for bits in [0o770, 0o707, 0o777] {
        definir_modo(&raiz, bits);
        let indice = IndiceLocal::em(raiz.join("novo").join(NOME_DO_ARQUIVO));
        indice.registrar_entrada_por_id(&instalacao(), true);
        assert!(!indice.ligado(), "{bits:o}");
        assert!(!raiz.join("novo").exists(), "{bits:o}");
    }
    definir_modo(&raiz, 0o700);
}

#[cfg(unix)]
#[test]
fn banco_existente_com_bits_de_grupo_desliga() {
    let raiz = pasta("banco-aberto");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    definir_modo(&caminho, 0o640);
    let indice = IndiceLocal::em(caminho);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
}

#[test]
fn caminho_respeita_sem_indice_arquivo_e_padrao() {
    let url = Path::new("/privado/ies/url-controle");
    let padrao = Some(PathBuf::from("/privado/ies/ies.sqlite3"));
    let os = |s: &'static str| Some(OsStr::new(s));
    assert_eq!(caminho_do_indice(None, None, url), padrao);
    assert_eq!(caminho_do_indice(os(""), None, url), padrao);
    assert_eq!(caminho_do_indice(os("0"), os(""), url), padrao);
    assert_eq!(caminho_do_indice(os("1"), None, url), None);
    assert_eq!(caminho_do_indice(os("sim"), os("/x/i.db"), url), None);
    assert_eq!(
        caminho_do_indice(None, os("/x/i.db"), url),
        Some(PathBuf::from("/x/i.db"))
    );
}

#[test]
fn desligado_nao_cria_nada_e_nao_responde() {
    let indice = IndiceLocal::desligado();
    indice.registrar_entrada_por_id(&instalacao(), true);
    indice.registrar_processo(&instalacao(), "N1", "10");
    assert!(!indice.ligado());
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert_eq!(indice.id_por_nup(&instalacao(), "N1"), None);
}

#[test]
fn arquivo_corrompido_desliga_sem_erro_e_sem_apagar() {
    let raiz = pasta("corrompido");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let lixo = vec![b'x'; 8192];
    fs::write(&caminho, &lixo).unwrap();
    definir_modo(&caminho, 0o600);
    let indice = IndiceLocal::em(caminho.clone());
    indice.registrar_entrada_por_id(&instalacao(), true);
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
    assert_eq!(fs::read(&caminho).unwrap(), lixo);
}

#[test]
fn versao_futura_desliga_sem_migrar() {
    let raiz = pasta("versao-futura");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    conexao.pragma_update(None, "user_version", 99).unwrap();
    drop(conexao);
    let indice = IndiceLocal::em(caminho.clone());
    assert_eq!(indice.entrada_por_id(&instalacao()), None);
    assert!(!indice.ligado());
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    let versao: i64 = conexao
        .pragma_query_value(None, "user_version", |l| l.get(0))
        .unwrap();
    assert_eq!(versao, 99);
}

#[test]
fn esquema_versionado_em_wal() {
    let raiz = pasta("esquema");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    let versao: i64 = conexao
        .pragma_query_value(None, "user_version", |l| l.get(0))
        .unwrap();
    let modo: String = conexao
        .pragma_query_value(None, "journal_mode", |l| l.get(0))
        .unwrap();
    assert_eq!((versao, modo.as_str()), (4, "wal"));
}

/// A corrida está na criação: duas conexões preparando o mesmo arquivo novo
/// (troca para WAL e migração). Repetida para não depender da sorte.
#[test]
fn duas_conexoes_concorrentes_gravam_sem_perder_linhas() {
    for rodada in 0..12 {
        concorrencia_numa_pasta_nova(&format!("concorrente-{rodada}"));
    }
}

/// Executado em subprocessos pelo teste abaixo. Sem as variáveis privadas de
/// teste, o harness o executa como teste vazio normalmente.
#[test]
fn grava_linhas_em_processo_auxiliar() {
    let (Ok(caminho), Ok(prefixo), Ok(base_id)) = (
        std::env::var("IES_TEST_SQLITE_CHILD_PATH"),
        std::env::var("IES_TEST_SQLITE_CHILD_PREFIX"),
        std::env::var("IES_TEST_SQLITE_CHILD_BASE_ID"),
    ) else {
        return;
    };
    let indice = IndiceLocal::em(PathBuf::from(caminho));
    let base_id: u32 = base_id.parse().unwrap();
    for n in 0..40 {
        indice.registrar_processo(
            &instalacao(),
            &format!("{prefixo}{n}"),
            &(base_id + n).to_string(),
        );
    }
    assert!(indice.ligado());
}

#[test]
fn dois_processos_gravam_no_mesmo_banco_sem_perder_linhas() {
    let raiz = pasta("processos-concorrentes");
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    IndiceLocal::em(caminho.clone()).registrar_entrada_por_id(&instalacao(), true);
    let executavel = std::env::current_exe().unwrap();
    let filhos: Vec<_> = [("A", "1000"), ("B", "2000")]
        .into_iter()
        .map(|(prefixo, base_id)| {
            Command::new(&executavel)
                .arg("grava_linhas_em_processo_auxiliar")
                .env("IES_TEST_SQLITE_CHILD_PATH", &caminho)
                .env("IES_TEST_SQLITE_CHILD_PREFIX", prefixo)
                .env("IES_TEST_SQLITE_CHILD_BASE_ID", base_id)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for filho in filhos {
        let saida = filho.wait_with_output().unwrap();
        assert!(
            saida.status.success(),
            "subprocesso SQLite falhou: {}{}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
    }

    let leitor = IndiceLocal::em(caminho);
    for n in 0..40 {
        assert_eq!(
            leitor.id_por_nup(&instalacao(), &format!("A{n}")),
            Some((1000 + n).to_string())
        );
        assert_eq!(
            leitor.id_por_nup(&instalacao(), &format!("B{n}")),
            Some((2000 + n).to_string())
        );
    }
}

fn concorrencia_numa_pasta_nova(nome: &str) {
    let raiz = pasta(nome);
    let caminho = raiz.join(NOME_DO_ARQUIVO);
    let tarefas: Vec<_> = ["A", "B"]
        .into_iter()
        .enumerate()
        .map(|(n, prefixo)| {
            let caminho = caminho.clone();
            std::thread::spawn(move || {
                let indice = IndiceLocal::em(caminho);
                for i in 0..40 {
                    let id = (n * 1000 + i).to_string();
                    indice.registrar_processo(&instalacao(), &format!("{prefixo}{i}"), &id);
                }
                indice.ligado()
            })
        })
        .collect();
    for tarefa in tarefas {
        assert!(tarefa.join().unwrap());
    }
    let leitor = IndiceLocal::em(caminho);
    for i in 0..40 {
        assert_eq!(
            leitor.id_por_nup(&instalacao(), &format!("A{i}")),
            Some(i.to_string())
        );
        let b = (1000 + i).to_string();
        assert_eq!(leitor.id_por_nup(&instalacao(), &format!("B{i}")), Some(b));
    }
}

#[test]
fn conflito_substitui_as_linhas_antigas() {
    let raiz = pasta("conflito");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    indice.registrar_processo(&sei, "A", "1");
    indice.registrar_processo(&sei, "B", "2");
    indice.registrar_processo(&sei, "A", "2");
    assert_eq!(indice.id_por_nup(&sei, "A").as_deref(), Some("2"));
    assert_eq!(indice.id_por_nup(&sei, "B"), None);
    indice.registrar_processo(&sei, "C", "2");
    assert_eq!(indice.id_por_nup(&sei, "A"), None);
    assert_eq!(indice.id_por_nup(&sei, "C").as_deref(), Some("2"));
    indice.esquecer_processo(&sei, "C");
    assert_eq!(indice.id_por_nup(&sei, "C"), None);
    assert!(indice.ligado());
}

#[test]
fn nup_opaco_comparado_exatamente_e_por_instalacao() {
    let raiz = pasta("nup-opaco");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    indice.registrar_processo(&sei, "23106.000001/2026-01", "7");
    indice.registrar_processo(&outra, "23106.000001/2026-01", "7");
    assert_eq!(indice.id_por_nup(&sei, "23106000001202601"), None);
    assert_eq!(indice.id_por_nup(&sei, "23106.000001/2026-01 "), None);
    indice.esquecer_processo(&outra, "23106.000001/2026-01");
    assert_eq!(
        indice.id_por_nup(&sei, "23106.000001/2026-01").as_deref(),
        Some("7")
    );
}

#[test]
fn mesmo_nup_e_id_ficam_isolados_entre_hosts() {
    let raiz = pasta("origem-outro-host");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    let outro_host = Instalacao::interpretar("https://outro.exemplo/sei").unwrap();
    indice.registrar_processo(&sei, "NUP-IGUAL", "71");
    indice.registrar_processo(&outro_host, "NUP-IGUAL", "82");

    assert_eq!(indice.id_por_nup(&sei, "NUP-IGUAL").as_deref(), Some("71"));
    assert_eq!(
        indice.id_por_nup(&outro_host, "NUP-IGUAL").as_deref(),
        Some("82")
    );
    assert_eq!(indice.nup_por_id(&sei, "82"), None);
    assert_eq!(indice.nup_por_id(&outro_host, "71"), None);
}

#[test]
fn identidade_contestada_pelo_servidor_deixa_de_ser_reutilizavel() {
    let raiz = pasta("servidor-contestou-identidade");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    indice.registrar_processo(&sei, "NUP-ALVO", "71");

    // O chamador abriu o id guardado e o servidor mostrou outro NUP.
    indice.esquecer_processo(&sei, "NUP-ALVO");

    assert_eq!(indice.id_por_nup(&sei, "NUP-ALVO"), None);
    assert_eq!(indice.nup_por_id(&sei, "71"), None);
}

fn relogio() -> (Arc<AtomicU64>, impl Fn() -> f64 + Send + 'static) {
    let agora = Arc::new(AtomicU64::new(1_800_000_000));
    let leitura = Arc::clone(&agora);
    #[allow(clippy::cast_precision_loss)]
    let ler = move || leitura.load(Ordering::SeqCst) as f64;
    (agora, ler)
}

const DIA: u64 = 24 * 3600;

#[test]
fn recusa_de_desconhecida_vale_na_primeira_e_expira_em_30_dias() {
    let raiz = pasta("expira");
    let (agora, ler) = relogio();
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO)).com_relogio(ler);
    let sei = instalacao();
    assert_eq!(indice.entrada_por_id(&sei), None);
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Recusada));
    agora.fetch_add(29 * DIA, Ordering::SeqCst);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Recusada));
    agora.fetch_add(2 * DIA, Ordering::SeqCst);
    assert_eq!(indice.entrada_por_id(&sei), None);
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Recusada));
}

#[test]
fn aceita_so_rebaixa_com_duas_recusas_seguidas() {
    let raiz = pasta("rebaixa");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let sei = instalacao();
    indice.registrar_entrada_por_id(&sei, true);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Aceita));
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Aceita));
    indice.registrar_entrada_por_id(&sei, true);
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Aceita));
    indice.registrar_entrada_por_id(&sei, false);
    assert_eq!(indice.entrada_por_id(&sei), Some(AceiteDaEntrada::Recusada));
    let outra = Instalacao::interpretar("https://sei.exemplo/outro").unwrap();
    assert_eq!(indice.entrada_por_id(&outra), None);
}

#[test]
fn aceite_nao_expira() {
    let raiz = pasta("aceite");
    let (agora, ler) = relogio();
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO)).com_relogio(ler);
    indice.registrar_entrada_por_id(&instalacao(), true);
    agora.fetch_add(400 * DIA, Ordering::SeqCst);
    assert_eq!(
        indice.entrada_por_id(&instalacao()),
        Some(AceiteDaEntrada::Aceita)
    );
}

#[test]
fn debug_do_indice_nao_mostra_caminho_nem_dados() {
    let raiz = pasta("debug-indice");
    let indice = IndiceLocal::em(raiz.join(NOME_DO_ARQUIVO));
    let texto = format!("{indice:?}");
    assert!(texto.contains("ligado"), "{texto}");
    assert!(!texto.contains("ies-indice"), "{texto}");
}
