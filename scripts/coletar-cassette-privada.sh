#!/usr/bin/env bash
# Handoff de captura em máquina autorizada. NUNCA publica nem saneia cassettes.
set -euo pipefail
umask 077
usage() {
  cat <<'EOF'
Uso: scripts/coletar-cassette-privada.sh --instalacao unb|ufrj|go|pen1|pen2|pen3|pen4
       --operacao processo.obter|documento.obter|unidade.listar_processos|processo.buscar|documento.buscar
       --cenario NOME --destino DIR [--executar --gravador BIN --autorizar-live
       --url-file CAMINHO --cookie-file CAMINHO --origem-file CAMINHO]
Por padrão apenas prévia, sem abrir arquivos nem conectar ao SEI.
--executar exige autorização externa prévia e um coletor próprio executável.
Dados brutos permanecem privados no destino, nunca são sanitizados/publicados.
EOF
}
inst='';operacao='';cenario='';destino='';gravador='';url_file='';cookie_file='';origem_file='';executar=0;autorizado=0
while (($#)); do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --instalacao|--operacao|--cenario|--destino|--gravador|--url-file|--cookie-file|--origem-file)
      (($# >= 2)) || { echo 'ERRO: argumento sem valor' >&2; exit 2; }
      case "$1" in
        --instalacao) inst=$2 ;; --operacao) operacao=$2 ;; --cenario) cenario=$2 ;;
        --destino) destino=$2 ;; --gravador) gravador=$2 ;;
        --url-file) url_file=$2 ;; --cookie-file) cookie_file=$2 ;; --origem-file) origem_file=$2 ;;
      esac
      shift 2 ;;
    --executar) executar=1; shift ;;
    --autorizar-live) autorizado=1; shift ;;
    *) echo 'ERRO: opção desconhecida' >&2; exit 2 ;;
  esac
done
case "$inst" in unb|ufrj|go|pen1|pen2|pen3|pen4) ;; *) echo 'ERRO: instalação inválida' >&2; exit 2 ;; esac
case "$operacao" in unidade.listar_processos|processo.obter|documento.obter|processo.buscar|documento.buscar) ;; *) echo 'ERRO: operação não permitida neste roteiro' >&2; exit 2 ;; esac
[[ "$cenario" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo 'ERRO: cenário inválido' >&2; exit 2; }
[[ -n "$destino" && "$destino" == /* ]] || { echo 'ERRO: destino deve ser absoluto' >&2; exit 2; }
if (( ! executar )); then
  printf 'NOT_RUN | instalação=%s operação=%s cenário=%s | prévia; nenhuma requisição\n' "$inst" "$operacao" "$cenario"
  exit 0
fi
(( autorizado )) || { echo 'ERRO: --autorizar-live obrigatório (somente após autorização externa real)' >&2; exit 2; }
[[ "$gravador" == /* && -x "$gravador" && ! -L "$gravador" ]] || { echo 'ERRO: gravador executável absoluto requerido' >&2; exit 2; }
for f in "$url_file" "$cookie_file" "$origem_file"; do
  [[ "$f" == /* && -f "$f" && ! -L "$f" && -s "$f" ]] || { echo 'ERRO: arquivo privado inválido' >&2; exit 2; }
  perm=$(stat -c %a -- "$f") || { echo 'ERRO: metadados indisponíveis' >&2; exit 2; }
  (( (8#$perm & 8#77) == 0 )) || { echo 'ERRO: arquivo privado com permissão de grupo/outros' >&2; exit 2; }
done
[[ ! -e "$destino" && ! -L "$destino" ]] || { echo 'ERRO: destino já existe; crie diretório novo' >&2; exit 2; }
mkdir -m 700 -p -- "$destino"
[[ -d "$destino" && ! -L "$destino" ]] || { echo 'ERRO: destino inválido' >&2; exit 2; }
pasta=$(mktemp -d "$destino/captura-${inst}-${cenario}.XXXXXXXX")
# O gravador recebe APENAS caminhos de credenciais por variáveis e diretório privado.
# Deve ser um componente autorizado independente; seus logs permanecem privados.
if IES_CAPTURE_INSTALLATION="$inst" IES_CAPTURE_OPERATION="$operacao" \
   IES_CAPTURE_SCENARIO="$cenario" IES_CAPTURE_OUTPUT_DIR="$pasta" \
   IES_URL_CONTROLE_FILE="$url_file" IES_COOKIE_FILE="$cookie_file" IES_ORIGEM_FILE="$origem_file" \
   "$gravador" >"$pasta/gravador.stdout" 2>"$pasta/gravador.stderr"; then
  status='coleta_privada_nao_sanitizada'
  rc=0
else
  status='falha_de_coleta_privada'
  rc=1
fi
# Manifesto não contém URLs, cookies, conteúdo, nomes de usuário nem paths secretos.
python3 - "$pasta" "$inst" "$operacao" "$cenario" "$status" <<'PY'
import hashlib,json,sys,pathlib
p=pathlib.Path(sys.argv[1])
arquivos=[]
for f in sorted(p.rglob('*')):
    if f.is_file() and not f.is_symlink() and f.name!='manifesto.json':
        arquivos.append({'arquivo':str(f.relative_to(p)),'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
manifesto={'schema_version':1,'instalacao':sys.argv[2],'operacao':sys.argv[3],
           'cenario':sys.argv[4],'estado':sys.argv[5],
           'publicacao':'proibida_ate_sanitizacao_e_revisao','arquivos_brutos':arquivos}
(p/'manifesto.json').write_text(json.dumps(manifesto,indent=2,ensure_ascii=False)+'\n')
(p/'NAO_PUBLICAR_DADOS_BRUTOS').write_text('Conferir efeitos e autorização, remover segredos e dados pessoais, validar hashes, só então publicar por processo separado.\n')
PY
printf '%s | instalação=%s operação=%s cenário=%s | saída privada em diretório da execução\n' "$status" "$inst" "$operacao" "$cenario"
exit "$rc"
