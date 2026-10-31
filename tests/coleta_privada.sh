#!/usr/bin/env bash
set -euo pipefail
raiz=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
script="$raiz/scripts/coletar-cassette-privada.sh"
temp=$(mktemp -d)
trap 'rm -rf -- "$temp"' EXIT
for nome in url cookie origem; do printf 'EXEMPLO_FICTICIO\n' >"$temp/$nome"; chmod 600 "$temp/$nome"; done
cat >"$temp/gravador" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "${IES_CAPTURE_INSTALLATION:-}" == 'unb' ]]
[[ "${IES_CAPTURE_OPERATION:-}" == 'documento.obter' ]]
[[ "${IES_CAPTURE_SCENARIO:-}" == 'documento-numero-e-busca-unb' ]]
[[ -d "$IES_CAPTURE_OUTPUT_DIR" ]]
[[ -f "$IES_COOKIE_FILE" && -f "$IES_URL_CONTROLE_FILE" && -f "$IES_ORIGEM_FILE" ]]
printf 'CASSSETTE_SIMULADA\n' >"$IES_CAPTURE_OUTPUT_DIR/dados-brutos.json"
printf 'TOKEN_PRIVADO_SIMULADO\n'
EOF
chmod +x "$temp/gravador"
args=(--instalacao unb --operacao documento.obter --cenario documento-numero-e-busca-unb --url-file "$temp/url" --cookie-file "$temp/cookie" --origem-file "$temp/origem" --gravador "$temp/gravador")
previa=$(bash "$script" --instalacao unb --operacao documento.obter --cenario documento-numero-e-busca-unb --destino "$temp/previa")
[[ "$previa" == NOT_RUN* && ! -e "$temp/previa" ]]
if bash "$script" "${args[@]}" --destino "$temp/sem-autorizacao" --executar >"$temp/nao.out" 2>&1; then echo 'ERRO: executou sem autorização' >&2; exit 1; fi
# RED: arquivo de saída já existente nunca pode sofrer chmod nem ser reutilizado.
mkdir "$temp/existente"
chmod 755 "$temp/existente"
if bash "$script" "${args[@]}" --destino "$temp/existente" --executar --autorizar-live >"$temp/existente.out" 2>&1; then
  echo 'ERRO: destino preexistente foi aceito' >&2; exit 1
fi
[[ $(stat -c %a "$temp/existente") == 755 ]]
saida=$(bash "$script" "${args[@]}" --destino "$temp/nova-captura" --executar --autorizar-live)
[[ "$saida" == coleta_privada_nao_sanitizada* ]]
python3 - "$temp/nova-captura" <<'PY'
import json,sys,pathlib,hashlib
raiz=pathlib.Path(sys.argv[1]);pastas=list(raiz.glob('captura-*'))
assert len(pastas)==1
p=pastas[0]
assert (p/'NAO_PUBLICAR_DADOS_BRUTOS').exists()
m=json.loads((p/'manifesto.json').read_text())
assert m['estado']=='coleta_privada_nao_sanitizada'
assert m['instalacao']=='unb' and m['cenario']=='documento-numero-e-busca-unb'
assert m['publicacao']=='proibida_ate_sanitizacao_e_revisao'
assert next(x['sha256'] for x in m['arquivos_brutos'] if x['arquivo']=='dados-brutos.json')==hashlib.sha256(b'CASSSETTE_SIMULADA\n').hexdigest()
assert not any('TOKEN' in str(v) or 'EXEMPLO_FICTICIO' in str(v) for v in m.values())
assert 'TOKEN_PRIVADO_SIMULADO' in (p/'gravador.stdout').read_text()
PY
# Permissões de arquivo de cookie inseguras impedem qualquer chamada.
chmod 644 "$temp/cookie"
if bash "$script" "${args[@]}" --destino "$temp/inseguro" --executar --autorizar-live >"$temp/inseguro.out" 2>&1; then echo 'ERRO: modo 644 aceito' >&2; exit 1; fi
[[ ! -e "$temp/inseguro" ]]
# Coletor externo não pode fazer uma captura com link simbólico parecer íntegra.
chmod 600 "$temp/cookie"
cat >"$temp/gravador-link" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
ln -s "$IES_COOKIE_FILE" "$IES_CAPTURE_OUTPUT_DIR/cookie-link"
EOF
chmod +x "$temp/gravador-link"
if bash "$script" "${args[@]}" --gravador "$temp/gravador-link" --destino "$temp/saida-link" --executar --autorizar-live >"$temp/link.out" 2>&1; then
  echo 'ERRO: captura com link simbólico foi certificada' >&2; exit 1
fi
[[ ! -s "$temp/link.out" || $(cat "$temp/link.out") != coleta_privada* ]]
echo 'PASS: prévia sem rede, recusa autorização ausente/destino reutilizado/permissão insegura, captura privada simulada e manifesto sem segredos'
