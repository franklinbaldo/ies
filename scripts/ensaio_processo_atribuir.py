"""Ensaio MANUAL #264: atribuição e remoção únicas em TREINAMENTO.

Registro privado reservado com O_EXCL ANTES do primeiro --confirmar; nunca
apagado automaticamente. Sem chamadas no CI e sem retry de escrita.
A CLI pública continua recusando --confirmar até a prova live autorizada.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

from ensaio_processo_criar import (
    chamar_ies, origem_privada_permitida, origens_de_treinamento,
)


def quantidade(resposta: dict) -> int | None:
    valor = resposta.get("requisicoes")
    return valor if type(valor) is int and valor > 0 else None


def registrar_estado_relido_incerto(relatorio: dict, resposta: dict, usuario: str) -> None:
    """Relato da CLI, não prova de que este POST produziu a alteração.

    Aceita apenas o usuário pedido, ausência explicitamente observada ou
    observação nula; nunca publica texto de resposta arbitrário no registro.
    """
    if "atribuicao_relida" not in resposta:
        return
    relida = resposta["atribuicao_relida"]
    if relida is None or (type(relida) is str and relida in ("", usuario)):
        relatorio["atribuicao_relida"] = relida


def atribuicao_da_unidade(resposta: dict, unidade: str) -> str | None:
    """None: não observado; string vazia: ausência observada."""
    unidades = resposta.get("aberto_em")
    if not isinstance(unidades, list):
        return None
    achados = [u for u in unidades if isinstance(u, dict) and u.get("sigla") == unidade]
    if len(achados) != 1 or "atribuido_a" not in achados[0]:
        return None
    pessoa = achados[0]["atribuido_a"]
    if pessoa is None:
        return ""
    if isinstance(pessoa, dict) and isinstance(pessoa.get("sigla"), str):
        return pessoa["sigla"]
    return None


def sincronizar_diretorio(pasta: Path) -> None:
    """Persistir a entrada de nome do registro, não só seus bytes.

    Se a plataforma não permite fsync em diretórios, falhar fechado:
    este executor manual não deve confirmar escrita sem trava durável.
    """
    if not hasattr(os, "O_DIRECTORY"):
        raise OSError("fsync de diretório indisponível")
    fd_diretorio = os.open(pasta, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd_diretorio)
    finally:
        os.close(fd_diretorio)


def ensaiar(*, ies: str, registro: Path, nup: str, usuario: str, origem: str,
            executar=chamar_ies, pausa=time.sleep) -> tuple[int, dict]:
    """Prévia → 1 POST → leitura → prévia remoção → 1 POST → leitura.

    Exige processo inicialmente sem atribuição: só assim --remover pode
    restaurar o estado inicial. Não repetir se qualquer etapa for incerta.
    """
    if registro.exists() or registro.is_symlink():
        return 2, {"estado": "BLOQUEADO", "motivo": "registro_ja_existente"}
    if not nup or not usuario or not origem or any(c.isspace() for c in nup + usuario):
        return 2, {"estado": "BLOQUEADO", "motivo": "argumentos_invalidos"}
    atribuir = ["processo", "atribuir", nup, "--usuario", usuario]
    remover = ["processo", "atribuir", nup, "--remover"]
    ler = ["processo", nup, "--sem-documentos", "--sem-andamentos",
           "--sem-marcadores", "--sem-propriedades"]
    intervalo = 95 if origem == "https://seitreinamento.go.gov.br/sei" else (
        60 if "tramita.processoeletronico.gov.br" in origem else 30
    )
    try:
        codigo, previa = executar(ies, atribuir)
    except (OSError, subprocess.TimeoutExpired, ValueError):
        return 1, {"estado": "PREVIA_FAIL", "requisicoes": [None]}
    identificador = previa.get("id_procedimento")
    unidade = previa.get("unidade")
    if (codigo != 0 or previa.get("nup") != nup
            or not isinstance(identificador, str) or not identificador
            or not isinstance(unidade, str) or not unidade
            or previa.get("usuario_pedido") != usuario
            or previa.get("usuario_atual") != ""
            or previa.get("confirmado") is not False
            or previa.get("alterado") is not False
            or quantidade(previa) is None):
        return 1, {"estado": "PREVIA_FAIL", "requisicoes": [quantidade(previa)]}

    # Trava durável antes do primeiro POST; segundo ensaio exige outro registro.
    try:
        fd = os.open(registro, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return 2, {"estado": "BLOQUEADO", "motivo": "registro_concorrente"}
    except OSError:
        return 2, {"estado": "BLOQUEADO", "motivo": "registro_indisponivel"}
    relatorio = {
        "estado": "CONFIRMACAO_INICIADA", "origem": origem, "nup": nup,
        "unidade": unidade, "usuario": usuario, "residuo": "possivel_atribuicao",
        "requisicoes": [quantidade(previa)],
    }
    confirmacao_iniciada = False

    def gravar(*, atomico: bool = False) -> None:
        dados = (json.dumps(relatorio, ensure_ascii=False) + "\n").encode()

        def escrever(inode: int) -> None:
            restante = memoryview(dados)
            while restante:
                escritos = os.write(inode, restante)
                if escritos <= 0:
                    raise OSError("checkpoint incompleto")
                restante = restante[escritos:]
            os.fsync(inode)

        if not atomico:
            os.ftruncate(fd, 0)
            os.lseek(fd, 0, os.SEEK_SET)
            escrever(fd)
            return

        # O estado final só substitui o registro depois de fsync completo.
        # Se fsync falhar, o checkpoint anterior ainda indica efeito possível.
        temporario_fd, temporario = tempfile.mkstemp(
            prefix=f".{registro.name}.", dir=registro.parent
        )
        try:
            escrever(temporario_fd)
            os.close(temporario_fd)
            temporario_fd = -1
            os.replace(temporario, registro)
            sincronizar_diretorio(registro.parent)
        finally:
            if temporario_fd != -1:
                os.close(temporario_fd)
            if os.path.exists(temporario):
                os.unlink(temporario)

    def etapa(operacao: list[str]) -> tuple[int, dict]:
        try:
            codigo, resposta = executar(ies, operacao)
        except (OSError, subprocess.TimeoutExpired, ValueError):
            codigo, resposta = 3, {}
        relatorio["requisicoes"].append(quantidade(resposta))
        return codigo, resposta

    try:
        gravar()
        # fsync(fd) não garante que o nome criado com O_EXCL sobreviva
        # à queda do host. Nenhum POST ocorre antes do fsync do diretório.
        sincronizar_diretorio(registro.parent)
        pausa(intervalo)
        confirmacao_iniciada = True
        codigo, escrita = etapa(atribuir + ["--confirmar"])
        if (codigo != 0 or escrita.get("nup") != nup
                or escrita.get("id_procedimento") != identificador
                or escrita.get("unidade") != unidade
                or escrita.get("usuario_pedido") != usuario
                or escrita.get("confirmado") is not True
                or escrita.get("alterado") is not True
                or quantidade(escrita) is None):
            relatorio["estado"] = "EFEITO_INCERTO"
            registrar_estado_relido_incerto(relatorio, escrita, usuario)
            gravar()
            return 3, relatorio
        relatorio["estado"] = "ATRIBUICAO_VERIFICACAO_PENDENTE"
        gravar()
        pausa(intervalo)
        codigo, lido = etapa(ler)
        if (codigo != 0 or lido.get("numero") != nup
                or lido.get("id_procedimento") != identificador
                or quantidade(lido) is None
                or atribuicao_da_unidade(lido, unidade) != usuario):
            relatorio["estado"] = "VERIFICACAO_INCONCLUSIVA"
            gravar()
            return 3, relatorio
        relatorio["residuo"] = "atribuido_ao_usuario_do_ensaio"
        relatorio["estado"] = "ATRIBUICAO_VERIFICADA"
        gravar()
        pausa(intervalo)
        codigo, previa_remocao = etapa(remover)
        if (codigo != 0 or previa_remocao.get("nup") != nup
                or previa_remocao.get("id_procedimento") != identificador
                or previa_remocao.get("unidade") != unidade
                or previa_remocao.get("usuario_atual") != usuario
                or previa_remocao.get("usuario_pedido") is not None
                or previa_remocao.get("confirmado") is not False
                or previa_remocao.get("alterado") is not False
                or quantidade(previa_remocao) is None):
            relatorio["estado"] = "REMOCAO_BLOQUEADA"
            gravar()
            return 3, relatorio
        relatorio["estado"] = "REMOCAO_INICIADA"
        relatorio["residuo"] = "possivel_remocao"
        gravar()
        pausa(intervalo)
        codigo, remocao = etapa(remover + ["--confirmar"])
        if (codigo != 0 or remocao.get("nup") != nup
                or remocao.get("id_procedimento") != identificador
                or remocao.get("unidade") != unidade
                or remocao.get("usuario_pedido") is not None
                or remocao.get("confirmado") is not True
                or remocao.get("alterado") is not True
                or quantidade(remocao) is None):
            relatorio["estado"] = "EFEITO_INCERTO"
            registrar_estado_relido_incerto(relatorio, remocao, usuario)
            gravar()
            return 3, relatorio
        relatorio["estado"] = "REMOCAO_VERIFICACAO_PENDENTE"
        gravar()
        pausa(intervalo)
        codigo, lido = etapa(ler)
        if (codigo != 0 or lido.get("numero") != nup
                or lido.get("id_procedimento") != identificador
                or quantidade(lido) is None
                or atribuicao_da_unidade(lido, unidade) != ""):
            relatorio["estado"] = "REMOCAO_INCONCLUSIVA"
            gravar()
            return 3, relatorio
        relatorio["estado"] = "PASS"
        relatorio["residuo"] = "sem_atribuicao"
        gravar(atomico=True)
        return 0, relatorio
    except OSError:
        if not confirmacao_iniciada:
            return 2, {"estado": "BLOQUEADO",
                       "motivo": "registro_indisponivel_antes_do_post",
                       "requisicoes": relatorio["requisicoes"]}
        # O registro não pode mais ser atualizado; a trava O_EXCL permanece.
        # Não repetir POST nem tentar recuperar a persistência por novo write.
        relatorio["estado"] = "EFEITO_INCERTO"
        relatorio["motivo"] = "registro_indisponivel_apos_post"
        if relatorio["residuo"] == "sem_atribuicao":
            # PASS só é durável depois do fsync; não publicar resíduo limpo.
            relatorio["residuo"] = "possivel_remocao"
        return 3, relatorio
    finally:
        os.close(fd)


def main() -> int:
    parser = argparse.ArgumentParser(description="Ensaio #264: uma vez, só treinamento")
    parser.add_argument("--ies", default="ies")
    parser.add_argument("--registro", type=Path)
    parser.add_argument("--nup")
    parser.add_argument("--usuario")
    parser.add_argument("--preflight", action="store_true",
                        help="confere a origem e não abre rede")
    args = parser.parse_args()
    try:
        permitidas = origens_de_treinamento(
            Path(__file__).resolve().parents[1] /
            "docs/interfaces/credenciais-de-treinamento.md"
        )
        origem = origem_privada_permitida(permitidas)
    except (OSError, ValueError, KeyError):
        codigo, relatorio = 2, {"estado": "BLOQUEADO",
                              "motivo": "origem_nao_verificada"}
    else:
        if args.preflight:
            codigo, relatorio = 0, {"estado": "PREFLIGHT_PASS", "origem": origem}
        elif args.registro is None or args.nup is None or args.usuario is None:
            codigo, relatorio = 2, {"estado": "BLOQUEADO", "motivo": "argumentos_ausentes"}
        else:
            codigo, relatorio = ensaiar(
                ies=args.ies, registro=args.registro, nup=args.nup,
                usuario=args.usuario, origem=origem,
            )
    print(json.dumps(relatorio, ensure_ascii=False))
    return codigo


if __name__ == "__main__":
    raise SystemExit(main())
