---
type: CommandContract
title: Inclusão de documento externo PDF — inspeção local antes da escrita
operation: documento.incluir-externo
command: ies documento incluir-externo --processo NUP --arquivo ARQUIVO.pdf --tipo TIPO [--confirmar]
effect: write-document-external
implementation: partial
---

# Incluir documento externo em processo — #332

A capacidade está **PARCIAL LOCAL**: `ies documento incluir-externo --processo NUP --arquivo ARQUIVO.pdf --tipo TIPO` inspeciona **somente o arquivo local**, não a disponibilidade do formulário remoto. É uma etapa de preparação, **não uma prévia do SEI**. Não abre sessão, não consulta o processo, não oferece lista de tipos e não transmite o arquivo. A flag `--confirmar` é reconhecida, mas **recusa antes de sessão e rede** (`cadastro_externo_ainda_nao_validado`) até comprovar a operação completa.

## Contrato atual do núcleo local

- O NUP é validado pelo tipo `NumeroProcesso`. O tipo solicitado é texto opaco não vazio, sem caracteres de controle; não é declarado como existente no SEI.
- O arquivo tem nome UTF-8 e extensão `.pdf` sem distinção de caixa, é arquivo regular, não é link simbólico (inclusive ancestrais na política desta operação), não é vazio e começa com assinatura `%PDF-`. **A assinatura não comprova que o documento é um PDF estruturalmente íntegro**; o SEI continua responsável por aceitar o conteúdo.
- O SHA-256 e o tamanho são calculados lendo o arquivo em blocos, sem carregar inteiro em memória. O limite só é aplicado quando **observado no cadastro**; a inspeção da CLI ainda não inventa um teto de upload.
- Saída `--json` de sucesso: `escopo:"inspecao_local"`, `processo`, `tipo_solicitado`, `arquivo:{nome,tamanho_bytes,sha256}`, `cadastro_sei_observado:false`, `previa_remota:false`, `confirmado:false`, `requisicoes:0`, `escrita_remota:false`. O conteúdo do arquivo e credenciais jamais são publicados.
- Qualquer falha de entrada: saída 2, `codigo:"entrada_invalida"`, `retentavel:false` e nenhum acesso à sessão/rede.

## Subcódigos da inspeção local

| Subcódigo | Fato observado |
| --- | --- |
| `arquivo_pdf_ausente` | O caminho informado não existe |
| `arquivo_pdf_nao_regular` | O destino é diretório ou outro objeto não regular |
| `arquivo_pdf_link_simbolico` | Destino ou ancestral do caminho é link simbólico |
| `arquivo_pdf_vazio` | Arquivo sem bytes |
| `extensao_pdf_invalida` | O nome não tem extensão PDF |
| `conteudo_nao_eh_pdf` | Os bytes iniciais não têm assinatura PDF |
| `arquivo_pdf_acima_do_limite` | O arquivo ultrapassa limite efetivamente informado pela tela |
| `leitura_do_arquivo_pdf_falhou` | Erro de abertura/leitura ou mudança de tamanho durante a leitura |
| `tipo_externo_invalido` | Tipo solicitado vazio ou com caractere de controle |
| `cadastro_externo_ainda_nao_validado` | Confirmação bloqueada até observação, cadastro e releitura |

## Escopo remoto ainda não implementado

O oráculo [Todos](https://github.com/franklinbaldo/todos/blob/main/src/todos/documento_externo.py) descreve na UnB de TREINAMENTO (11/10/2026) **duas etapas HTTP mutadoras diferentes sob uma única confirmação**: upload temporário multipart `documento_upload_anexo` → recibo → cadastro `documento_receber` com `hdnAnexos` → releitura independente da árvore e metadados do anexo. Não é correto exigir apenas um POST total, nem interpretar recibo de upload como documento já incluído.

A próxima fatia deve observar cada campo e action servidos; conferir os tipos e limites; usar `formulario_web`, origem e identidade; executar um único upload e um único cadastro, **sem retry**; distinguir `anexo confirmado`, `anexo não observado`, `documento criado` e `efeito incerto` com variantes tipadas, preferindo saída 3 a sucesso não provado. O caso [Todos #1246](https://github.com/franklinbaldo/todos/issues/1246) relata anexo somente visível na tela de alteração, não na árvore; **não** autoriza inferir ausência nem repetir POST.

**Evidência:** os testes sintéticos do núcleo estão em `src/documento_externo_incluir.rs`; live em treinamento: **NOT_RUN**. `--confirmar` permanece fechado.
